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
          observe-rust-vector-index! rust-vector-index)
  (import (except (chezscheme) errorf) (utils) (nanopass) (langs))

  ;; Fresh for one compiler invocation. Source objects are used only to join
  ;; circuit-pass observations to unique original nodes, never coordinates or
  ;; structural equality. Final lookup requires the exact original node.
  (define current-rust-vector-index-facts (make-parameter #f))
  (define collecting (make-parameter #f))
  (define constant-bindings (make-parameter #f))
  (define (exact-integer? value) (and (integer? value) (exact? value)))

  ;; A grammar key, not an evaluator. Numeric results come exclusively from
  ;; resolve-indices/simplify. Runtime formals, calls, conditionals and loop
  ;; bindings are deliberately not inferred to be local constants.
  (define (constant-shaped? expr)
    (define (arithmetic operator bits left right)
      (let ([left (constant-shaped? left)] [right (constant-shaped? right)])
        (and left right (list operator bits left right))))
    (nanopass-case (Lnodisclose Expression) expr
      [(quote ,src ,datum)
       (and (exact-integer? datum) (>= datum 0) (list 'literal datum))]
      [(var-ref ,src ,var-name)
       (and (eq-hashtable-ref (constant-bindings) var-name #f)
            (list 'variable var-name))]
      ;; Analysis widens unsigned operands before arithmetic. This is a
      ;; transparent typed coercion, not an independently evaluated constant.
      [(safe-cast ,src ,type ,type^ ,expr)
       (let ([shape (constant-shaped? expr)])
         (and shape (nanopass-case (Lnodisclose Type) type
              [(tunsigned ,src^ ,nat)
               (nanopass-case (Lnodisclose Type) type^
                 [(tunsigned ,src^^ ,nat^)
                  (and (>= nat nat^) (list 'widen nat nat^ shape))]
                 [else #f])]
              [else #f])))]
      [(+ ,src ,mbits ,expr1 ,expr2)
       (arithmetic '+ mbits expr1 expr2)]
      [(- ,src ,mbits ,expr1 ,expr2)
       (arithmetic '- mbits expr1 expr2)]
      [(* ,src ,mbits ,expr1 ,expr2)
       (arithmetic '* mbits expr1 expr2)]
      ;; Only the exact analysis-generated unsigned subtraction guard is pure
      ;; index grammar. Structural equality here checks that guard's operands;
      ;; occurrence authority still uses original node identity exclusively.
      [(seq ,src (assert ,src^ (>= ,src^^ ,bits ,expr1 ,expr2) ,mesg)
         (- ,src^^^ ,mbits ,expr3 ,expr4))
       (let ([left (constant-shaped? expr1)] [right (constant-shaped? expr2)])
         (and left right mbits
              (equal? mesg "result of subtraction would be negative")
              (= bits mbits)
              (equal? left (constant-shaped? expr3))
              (equal? right (constant-shaped? expr4))
              (list 'guarded-subtract mbits left right)))]
      [else #f]))

  (define-pass collect-vector-indices : Lnodisclose (ir) -> Lnodisclose ()
    (Expression : Expression (ir) -> Expression ()
      [(let* ,src ([,local* ,expr*] ...) ,expr)
       (parameterize ([constant-bindings (hashtable-copy (constant-bindings) #t)])
         (for-each
           (lambda (local value)
             (Expression value)
             (nanopass-case (Lnodisclose Argument) local
               [(,var-name ,type)
                (eq-hashtable-set! (constant-bindings) var-name (constant-shaped? value))]))
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
           (eq-hashtable-set! by-node ir 'unobserved)))
       (Expression expr)
       (Expression index)
       ir]))

  (define (make-rust-vector-index-facts ir)
    (let ([facts (vector (make-eq-hashtable) (make-eq-hashtable))])
      (parameterize ([collecting facts] [constant-bindings (make-eq-hashtable)])
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
)
