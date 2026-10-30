#!chezscheme

;;; This file is part of Compact.
;;; Copyright (C) 2026 Midnight Foundation
;;; SPDX-License-Identifier: Apache-2.0
;;; Licensed under the Apache License, Version 2.0 (the "License");
;;; you may not use this file except in compliance with the License.
;;; You may obtain a copy of the License at
;;;
;;;     http://www.apache.org/licenses/LICENSE-2.0
;;;
;;; Unless required by applicable law or agreed to in writing, software
;;; distributed under the License is distributed on an "AS IS" BASIS,
;;; WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
;;; See the License for the specific language governing permissions and
;;; limitations under the License.

(library (rust-vector-indices)
  (export current-rust-vector-index-facts make-rust-vector-index-facts
          observe-rust-vector-index! rust-vector-index rust-unsigned-maximum
          observe-rust-vector-index-guard! rust-vector-index-guard-state)
  (import (except (chezscheme) errorf) (utils) (nanopass) (langs))

  ;; Fresh for one compiler invocation. Source objects are used only to join
  ;; circuit-pass observations to unique original nodes, never coordinates or
  ;; structural equality. Final lookup requires the exact original node.
  (define current-rust-vector-index-facts (make-parameter #f))
  (define collecting (make-parameter #f))
  (define constant-bindings (make-parameter #f))
  (define index-guards (make-parameter '()))
  (define (exact-integer? value) (and (integer? value) (exact? value)))

  ;; These are authoritative analyzed type maxima, not evaluated expressions.
  (define (rust-unsigned-maximum type)
    (nanopass-case (Lnodisclose Type) type
      [(tunsigned ,src ,nat) (and (exact-integer? nat) (>= nat 0) nat)]
      [(talias ,src ,nominal? ,type-name ,type) (rust-unsigned-maximum type)]
      [else #f]))

  (define (bind-constant! local value)
    (nanopass-case (Lnodisclose Argument) local
      [(,var-name ,type)
       (let ([shape (constant-shaped? value)] [maximum (rust-unsigned-maximum type)])
         (eq-hashtable-set! (constant-bindings) var-name
           (and shape maximum (cons maximum shape)))
         (and shape maximum (list var-name maximum shape)))]))

  ;; Subtraction's analyzer binds nontrivial operands before its guard. Preserve
  ;; each base variable's identity even when two constants have equal values.
  (define (constant-base expr)
    (nanopass-case (Lnodisclose Expression) expr
      [(quote ,src ,datum)
       (and (exact-integer? datum) (>= datum 0)
            (cons datum (list 'literal datum)))]
      [(var-ref ,src ,var-name)
       (let ([info (eq-hashtable-ref (constant-bindings) var-name #f)])
         (and info (cons (car info) (list 'variable var-name))))]
      [else #f]))

  (define (subtraction-base expr)
    (nanopass-case (Lnodisclose Expression) expr
      [(safe-cast ,src ,type ,type^ ,expr)
       (let ([base (constant-base expr)] [maximum (rust-unsigned-maximum type)]
             [source-maximum (rust-unsigned-maximum type^)])
         (and base maximum source-maximum (= source-maximum (car base)) base))]
      [else (constant-base expr)]))

  (define (unsigned-bound-operand? expr maximum base)
    (nanopass-case (Lnodisclose Expression) expr
      [(safe-cast ,src ,type ,type^ ,expr)
       (let ([target (rust-unsigned-maximum type)] [source (rust-unsigned-maximum type^)])
         (and target source (= target maximum) (= source (car base))
              (equal? (constant-base expr) base)))]
      [else (and (= maximum (car base)) (equal? (constant-base expr) base))]))

  (define (guarded-subtraction bits mbits expr1 expr2 expr3 expr4 mesg)
    (let ([left (subtraction-base expr3)] [right (subtraction-base expr4)])
      (and left right (exact-integer? bits) (> bits 0)
           (exact-integer? mbits) (> mbits 0)
           (equal? mesg "result of subtraction would be negative")
           ;; Match arithmetic-binop in analysis-passes: comparison widens to
           ;; the larger original type, subtraction uses the left result type.
           ;; Match both annotations without evaluating either operand.
           (= mbits (max 1 (integer-length (car left))))
           (= bits (max 1 (integer-length (max (car left) (car right)))))
           (let ([maximum (max (car left) (car right))])
             (and (unsigned-bound-operand? expr1 maximum left)
                  (unsigned-bound-operand? expr2 maximum right)
                  (unsigned-bound-operand? expr3 (car left) left)
                  (unsigned-bound-operand? expr4 (car left) right)
                  (list 'guarded-subtract bits mbits (cdr left) (cdr right)))))))

  ;; A grammar key, not an evaluator. Numeric results come exclusively from
  ;; resolve-indices/simplify. Runtime formals, calls, conditionals and loop
  ;; bindings are deliberately not inferred to be local constants.
  (define (constant-shaped? expr)
    (define (arithmetic operator bits left right)
      (let ([left (constant-shaped? left)] [right (constant-shaped? right)])
        (and (exact-integer? bits) (> bits 0) left right
             (list operator bits left right))))
    (nanopass-case (Lnodisclose Expression) expr
      [(quote ,src ,datum)
       (and (exact-integer? datum) (>= datum 0) (list 'literal datum))]
      [(var-ref ,src ,var-name)
       (and (eq-hashtable-ref (constant-bindings) var-name #f)
            (list 'variable var-name))]
      [(let* ,src ([,local* ,expr*] ...) ,expr)
       (parameterize ([constant-bindings (hashtable-copy (constant-bindings) #t)])
         (let loop ([locals local*] [values expr*] [shapes '()])
           (if (null? locals)
               (let ([body (constant-shaped? expr)])
                 (and body (list 'let* (reverse shapes) body)))
               (let ([shape (bind-constant! (car locals) (car values))])
                 (and shape (loop (cdr locals) (cdr values) (cons shape shapes)))))))]
      ;; Analysis annotates unsigned arithmetic operands with the result type,
      ;; which can narrow for multiplication by zero. drop-safe-casts erases
      ;; these annotations before authoritative evaluation. Checked source
      ;; downcasts are distinct nodes and remain outside this grammar.
      [(safe-cast ,src ,type ,type^ ,expr)
       (let ([shape (constant-shaped? expr)]
             [maximum (rust-unsigned-maximum type)]
             [source-maximum (rust-unsigned-maximum type^)])
         (and shape maximum source-maximum
              (list 'unsigned-annotation maximum source-maximum shape)))]
      [(+ ,src ,mbits ,expr1 ,expr2)
       (arithmetic '+ mbits expr1 expr2)]
      [(* ,src ,mbits ,expr1 ,expr2)
       (arithmetic '* mbits expr1 expr2)]
      ;; Only the canonical analyzed unsigned subtraction guard is index grammar.
      ;; Operand identity and type bounds check its casts/widths independently;
      ;; occurrence authority still uses original node identity exclusively.
      [(seq ,src (assert ,src^ (>= ,src^^ ,bits ,expr1 ,expr2) ,mesg)
         (- ,src^^^ ,mbits ,expr3 ,expr4))
       (guarded-subtraction bits mbits expr1 expr2 expr3 expr4 mesg)]
      [else #f]))

  ;; Retain only guards syntactically inside this index. Guards in an outer
  ;; constant binding already execute at that binding's original location.
  (define-pass collect-index-guards : (Lnodisclose Expression) (ir) -> (Lnodisclose Expression) ()
    (Expression : Expression (ir) -> Expression ()
      ;; Explicit calls are required: generated processor argument evaluation
      ;; order is not a source-language evaluation-order guarantee.
      [(+ ,src ,mbits ,expr1 ,expr2) (Expression expr1) (Expression expr2) ir]
      [(- ,src ,mbits ,expr1 ,expr2) (Expression expr1) (Expression expr2) ir]
      [(* ,src ,mbits ,expr1 ,expr2) (Expression expr1) (Expression expr2) ir]
      [(safe-cast ,src ,type ,type^ ,expr) (Expression expr) ir]
      [(seq ,src ,expr* ... ,expr)
       (for-each Expression expr*) (Expression expr) ir]
      [(let* ,src ([,local* ,expr*] ...) ,expr)
       (for-each Expression expr*) (Expression expr) ir]
      [(assert ,src ,expr ,mesg)
       (Expression expr)
       (index-guards (cons (cons src mesg) (index-guards)))
       ir]))

  ;; Count every original assertion before selecting index guards. An assert
  ;; outside an index must not lend an observation to an index guard with the
  ;; same source object, even when only one copy reaches the circuit evaluator.
  (define-pass collect-assert-sources : Lnodisclose (ir) -> Lnodisclose ()
    (Expression : Expression (ir) -> Expression ()
      [(assert ,src ,[expr] ,mesg)
       (let ([observations (vector-ref (collecting) 3)])
         (eq-hashtable-set! observations src
           (if (eq-hashtable-contains? observations src) 'ambiguous 'unobserved)))
       ir]))

  (define-pass collect-vector-indices : Lnodisclose (ir) -> Lnodisclose ()
    (Expression : Expression (ir) -> Expression ()
      [(let* ,src ([,local* ,expr*] ...) ,expr)
       (parameterize ([constant-bindings (hashtable-copy (constant-bindings) #t)])
         (for-each
           (lambda (local value)
             (Expression value)
             (bind-constant! local value))
           local* expr*)
         (Expression expr))
       ir]
      [(vector-ref ,src ,type ,expr ,index)
       (let ([by-source (vector-ref (collecting) 0)]
             [by-node (vector-ref (collecting) 1)])
         ;; Even an ineligible second node makes the source join ambiguous.
         (when (source-object? src)
           (eq-hashtable-set! by-source src
             (if (eq-hashtable-contains? by-source src) 'ambiguous ir)))
         (when (constant-shaped? index)
           (eq-hashtable-set! by-node ir 'unobserved)
           (parameterize ([index-guards '()])
             (collect-index-guards index)
             (eq-hashtable-set! (vector-ref (collecting) 2) ir
               (reverse (index-guards))))))
       (Expression expr)
       (Expression index)
       ir]))

  (define (make-rust-vector-index-facts ir)
    (let ([facts (vector (make-eq-hashtable) (make-eq-hashtable)
                        (make-eq-hashtable) (make-eq-hashtable))])
      (parameterize ([collecting facts] [constant-bindings (make-eq-hashtable)])
        (collect-assert-sources ir)
        (collect-vector-indices ir))
      facts))

  (define (observe-rust-vector-index! src index)
    (let ([facts (current-rust-vector-index-facts)])
      (when facts
        (let* ([node (eq-hashtable-ref (vector-ref facts 0) src #f)]
               [by-node (vector-ref facts 1)]
               [old (and node (eq-hashtable-ref by-node node #f))])
          (when old
            (eq-hashtable-set! by-node node
              (cond
                [(eq? old 'unobserved) index]
                [(and (exact-integer? old) (= old index)) old]
                [else 'ambiguous])))))))

  (define (rust-vector-index expr)
    (let* ([facts (current-rust-vector-index-facts)]
           [index (and facts (eq-hashtable-ref (vector-ref facts 1) expr #f))])
      (and (exact-integer? index) (>= index 0) index)))

  ;; The existing circuit evaluator supplies these truth facts. This bridge
  ;; never evaluates a comparison or arithmetic expression itself. Conflicting
  ;; copies cannot turn a failing/unknown guard into successful evaluation.
  (define (observe-rust-vector-index-guard! src truth)
    (let ([facts (current-rust-vector-index-facts)])
      (when facts
        (let* ([guards (vector-ref facts 3)]
               [old (eq-hashtable-ref guards src #f)])
          (when old
            (eq-hashtable-set! guards src
              (cond [(eq? old 'unobserved) truth]
                    [(eq? old truth) old]
                    [else 'ambiguous])))))))

  (define (rust-vector-index-guard-state expr)
    (let* ([facts (current-rust-vector-index-facts)]
           [guards (and facts (eq-hashtable-ref (vector-ref facts 2) expr #f))])
      (if guards
          (let loop ([guards guards])
            (if (null? guards) 'safe
                (case (eq-hashtable-ref (vector-ref facts 3) (caar guards) 'unobserved)
                  [(true) (loop (cdr guards))]
                  [(false) (car guards)]
                  [else 'unknown])))
          'unknown)))
)
