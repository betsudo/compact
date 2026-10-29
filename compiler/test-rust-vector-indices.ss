#! /usr/bin/env -S scheme --compile-imported-libraries --program
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

;; Internal occurrence controls; generated consumer/proof gates remain separate.
(import (except (chezscheme) errorf) (utils) (nanopass) (langs)
        (parser) (frontend-passes) (analysis-passes) (pass-helpers)
        (rust-vector-indices))

(define collected '())
(define-pass collect-vector-index-nodes : Lnodisclose (ir) -> Lnodisclose ()
  (Expression : Expression (ir) -> Expression ()
    [(vector-ref ,src ,type ,expr ,index)
     (set! collected (cons ir collected))
     (Expression expr)
     (Expression index)
     ir]))

(define-pass duplicate : Lnodisclose (ir) -> Lnodisclose ()
  (Expression : Expression (ir) -> Expression ()
    [(vector-ref ,src ,type ,expr ,index)
     ;; Distinct nodes with the same exact source object must refuse.
     (let ([expr^ ir])
       `(seq ,src ,expr^ (vector-ref ,src ,type ,expr ,index)))]))

(define-pass duplicate-index-guards : Lnodisclose (ir) -> Lnodisclose ()
  (Expression : Expression (ir) -> Expression ()
    [(vector-ref ,src ,type ,expr ,index)
     `(vector-ref ,src ,type ,expr (+ ,src 3 ,index ,index))]))

(define-pass false-subtraction-guard : Lnodisclose (ir) -> Lnodisclose ()
  (Expression : Expression (ir) -> Expression ()
    [(seq ,src (assert ,src^ (>= ,src^^ ,bits ,expr1 ,expr2) ,mesg)
       (- ,src^^^ ,mbits ,expr3 ,expr4))
     ;; Leave the numeric subtraction untouched but make its guard false.
     ;; A known index must never erase this unrelated assertion.
     `(seq ,src (assert ,src^ (>= ,src^^ ,bits (quote ,src^^ 0) ,expr2) ,mesg)
        (- ,src^^^ ,mbits ,expr3 ,expr4))]))

(define-pass wrong-subtraction-guard-width : Lnodisclose (ir) -> Lnodisclose ()
  (Expression : Expression (ir) -> Expression ()
    [(seq ,src (assert ,src^ (>= ,src^^ ,bits ,expr1 ,expr2) ,mesg)
       (- ,src^^^ ,mbits ,expr3 ,expr4))
     `(seq ,src (assert ,src^ (>= ,src^^ ,(+ bits 1) ,expr1 ,expr2) ,mesg)
        (- ,src^^^ ,mbits ,expr3 ,expr4))]))

(define-pass wrong-subtraction-guard-message : Lnodisclose (ir) -> Lnodisclose ()
  (Expression : Expression (ir) -> Expression ()
    [(seq ,src (assert ,src^ (>= ,src^^ ,bits ,expr1 ,expr2) ,mesg)
       (- ,src^^^ ,mbits ,expr3 ,expr4))
     `(seq ,src (assert ,src^ (>= ,src^^ ,bits ,expr1 ,expr2) "unrelated assertion")
        (- ,src^^^ ,mbits ,expr3 ,expr4))]))

(define-pass checked-narrow-index : Lnodisclose (ir) -> Lnodisclose ()
  (Expression : Expression (ir) -> Expression ()
    [(vector-ref ,src ,type ,expr ,index)
     `(vector-ref ,src ,type ,expr
        (downcast-unsigned ,src 1 0 ,index))]))

(define-pass field-arithmetic-index : Lnodisclose (ir) -> Lnodisclose ()
  (Expression : Expression (ir) -> Expression ()
    [(vector-ref ,src ,type ,expr ,index)
     `(vector-ref ,src ,type ,expr
        (+ ,src #f (quote ,src 0) (quote ,src 1)))]))

(define-pass unguarded-subtraction-index : Lnodisclose (ir) -> Lnodisclose ()
  (Expression : Expression (ir) -> Expression ()
    [(vector-ref ,src ,type ,expr ,index)
     `(vector-ref ,src ,type ,expr
        (- ,src 1 (quote ,src 1) (quote ,src 0)))]))

(define (nodes ir)
  (set! collected '())
  (collect-vector-index-nodes ir)
  (reverse collected))

(define (analyze path)
  (define (run passes ir)
    (fold-left (lambda (ir pass) ((passrec-pass pass) ir)) ir passes))
  (parameterize ([contract-ht (make-hashtable symbol-hash eq?)]
                 [source-directory (path-parent path)]
                 [source-file-name (path-last (path-root path))]
                 [relative-path (path-parent path)])
    (run analysis-passes (run frontend-passes (run parser-passes path)))))

(define (source node)
  (nanopass-case (Lnodisclose Expression) node
    [(vector-ref ,src ,type ,expr ,index) src]))

(define guard-sources '())
(define-pass collect-guard-sources : Lnodisclose (ir) -> Lnodisclose ()
  (Expression : Expression (ir) -> Expression ()
    [(assert ,src ,[expr] ,mesg)
     (set! guard-sources (cons src guard-sources))
     ir]))

(define checks 0)
(define (check label result)
  (unless result (internal-errorf 'rust-vector-index-test "failed ~a" label))
  (set! checks (+ checks 1)))

(unless (= (length (command-line)) 2)
  (internal-errorf 'rust-vector-index-test "supply one absent temporary directory"))
(define output (cadr (command-line)))
(mkdir output)
(define (write-source name text)
  (let ([path (format "~a/~a.compact" output name)])
    (call-with-output-file path (lambda (port) (display text port)))
    path))

(define ir
  (analyze (write-source "alias"
    "export pure circuit choose(items: Vector<3, Field>): Field { const i = 1; return items[i]; }")))
(define node (car (nodes ir)))
(define src (source node))
(define facts (make-rust-vector-index-facts ir))
(parameterize ([current-rust-vector-index-facts facts])
  (check 'unobserved (not (rust-vector-index node)))
  (observe-rust-vector-index! src 1)
  (check 'exact-original (= (rust-vector-index node) 1))
  (observe-rust-vector-index! src 1)
  (check 'repeated-consistent (= (rust-vector-index node) 1))
  (let ([other-src (make-source-object (source-object-sfd src)
                     (source-object-bfp src) (source-object-efp src))])
    (observe-rust-vector-index! other-src 2)
    (check 'same-coordinates-distinct-source (= (rust-vector-index node) 1)))
  (let ([other-node
          (nanopass-case (Lnodisclose Expression) node
            [(vector-ref ,src ,type ,expr ,index)
             (with-output-language (Lnodisclose Expression)
               `(vector-ref ,src ,type ,expr ,index))])])
    (check 'distinct-node (not (eq? node other-node)))
    (check 'equal-node-cannot-borrow-fact (not (rust-vector-index other-node))))
  (observe-rust-vector-index! src 2)
  (check 'conflicting-copy (not (rust-vector-index node)))
  (observe-rust-vector-index! src 1)
  (check 'conflict-not-overwritten (not (rust-vector-index node))))
(check 'outside-invocation (not (rust-vector-index node)))
(parameterize ([current-rust-vector-index-facts (make-rust-vector-index-facts ir)])
  (check 'fresh-invocation (not (rust-vector-index node))))

(let* ([duplicate-ir (duplicate ir)] [duplicate-nodes (nodes duplicate-ir)])
  (check 'two-original-occurrences (= (length duplicate-nodes) 2))
  (parameterize ([current-rust-vector-index-facts (make-rust-vector-index-facts duplicate-ir)])
    (observe-rust-vector-index! (source (car duplicate-nodes)) 1)
    (check 'ambiguous-originals-refuse (for-all (lambda (node) (not (rust-vector-index node))) duplicate-nodes))))

(let* ([runtime-ir (analyze (write-source "runtime"
         "export pure circuit choose(items: Vector<3, Field>, i: Uint<8>): Field { return items[i]; }"))]
       [runtime-node (car (nodes runtime-ir))])
  (parameterize ([current-rust-vector-index-facts (make-rust-vector-index-facts runtime-ir)])
    (observe-rust-vector-index! (source runtime-node) 1)
    (check 'runtime-formal-not-a-local-constant (not (rust-vector-index runtime-node)))
    (observe-rust-vector-index! src 1)
    (check 'another-invocation-node (not (rust-vector-index node)))))

(let* ([subtraction-ir (analyze (write-source "subtraction"
         "export pure circuit choose(items: Vector<3, Field>): Field { const i = 2; return items[i - 1]; }"))]
       [subtraction-node (car (nodes subtraction-ir))]
       [forged-ir (false-subtraction-guard subtraction-ir)]
       [forged-node (car (nodes forged-ir))])
  (parameterize ([current-rust-vector-index-facts (make-rust-vector-index-facts subtraction-ir)])
    (observe-rust-vector-index! (source subtraction-node) 1)
    (check 'exact-constant-subtraction-guard (= (rust-vector-index subtraction-node) 1))
    (check 'numeric-fact-does-not-prove-guard (eq? (rust-vector-index-guard-state subtraction-node) 'unknown))
    (set! guard-sources '())
    (collect-guard-sources subtraction-ir)
    (let* ([guard-src (car guard-sources)]
           [copy-src (make-source-object (source-object-sfd guard-src)
                       (source-object-bfp guard-src) (source-object-efp guard-src))])
      (observe-rust-vector-index-guard! copy-src 'true)
      (check 'guard-coordinates-not-authority (eq? (rust-vector-index-guard-state subtraction-node) 'unknown))
      (observe-rust-vector-index-guard! guard-src 'false)
      (check 'failing-guard-retained (eq? (car (rust-vector-index-guard-state subtraction-node)) guard-src))
      (observe-rust-vector-index-guard! guard-src 'true)
      (check 'guard-conflict-not-erased (eq? (rust-vector-index-guard-state subtraction-node) 'unknown))
      (observe-rust-vector-index-guard! guard-src 'true)
      (check 'guard-conflict-not-overwritten (eq? (rust-vector-index-guard-state subtraction-node) 'unknown))))
  (parameterize ([current-rust-vector-index-facts (make-rust-vector-index-facts subtraction-ir)])
    (observe-rust-vector-index! (source subtraction-node) 1)
    (observe-rust-vector-index-guard! (car guard-sources) 'true)
    (check 'successful-guard-is-authoritative (eq? (rust-vector-index-guard-state subtraction-node) 'safe)))
  (let* ([copied-ir (duplicate-index-guards subtraction-ir)]
         [copied-node (car (nodes copied-ir))])
    (parameterize ([current-rust-vector-index-facts (make-rust-vector-index-facts copied-ir)])
      (observe-rust-vector-index! (source copied-node) 2)
      (observe-rust-vector-index-guard! (car guard-sources) 'true)
      (check 'duplicate-guard-source-not-authority
        (eq? (rust-vector-index-guard-state copied-node) 'unknown))))
  (parameterize ([current-rust-vector-index-facts (make-rust-vector-index-facts forged-ir)])
    (observe-rust-vector-index! (source forged-node) 1)
    (check 'unrelated-false-guard-not-erased (not (rust-vector-index forged-node))))
  (for-each
    (lambda (transform label)
      (let* ([forged (transform subtraction-ir)] [node (car (nodes forged))])
        (parameterize ([current-rust-vector-index-facts (make-rust-vector-index-facts forged)])
          (observe-rust-vector-index! (source node) 1)
          (check label (not (rust-vector-index node))))))
    (list wrong-subtraction-guard-width wrong-subtraction-guard-message)
    '(wrong-guard-width-not-erased wrong-guard-message-not-erased)))

(let* ([ordered-ir (analyze (write-source "guard-order"
         "export pure circuit choose(items: Vector<3, Field>): Field { const a = 1; const b = 2; const c = 3; return items[(a - b) * 0 + (a - c) * 0]; }"))]
       [ordered-node (car (nodes ordered-ir))])
  (set! guard-sources '())
  (collect-guard-sources ordered-ir)
  (let ([ordered-sources (reverse guard-sources)])
    (check 'two-ordered-guards (= (length ordered-sources) 2))
    (parameterize ([current-rust-vector-index-facts (make-rust-vector-index-facts ordered-ir)])
      (observe-rust-vector-index! (source ordered-node) 0)
      (for-each (lambda (src) (observe-rust-vector-index-guard! src 'false)) ordered-sources)
      (check 'first-index-failure-source-retained
        (eq? (car (rust-vector-index-guard-state ordered-node)) (car ordered-sources))))))

(for-each
  (lambda (row)
    (let* ([label (car row)] [body (cadr row)] [expected (caddr row)]
           [ir (analyze (write-source (symbol->string label)
                 (string-append "type Idx = Uint<8>; type Nested = Idx; export pure circuit choose(items: Vector<3, Field>): Field { " body " }")))]
           [node (car (nodes ir))])
      (parameterize ([current-rust-vector-index-facts (make-rust-vector-index-facts ir)])
        (observe-rust-vector-index! (source node) expected)
        (let ([actual (rust-vector-index node)])
          (check label (and actual (= actual expected)))))))
  '((compound-left-multiply "const index = 1; return items[index * 2 - 1];" 1)
    (compound-left-add "const index = 1; return items[(index + 1) - 1];" 1)
    (compound-right "const a = 2; const one = 1; return items[a - (one * 1)];" 1)
    (wide-right "const a = 2; const one: Uint<8> = 1; return items[a - one];" 1)
    (nested-wide "const index = 1; const one: Uint<8> = 1; return items[(index + 1) - one - one];" 0)
    (direct-zero "const i = 1; return items[i * 0];" 0)
    (alias-zero "const i = 1; const index = i * 0; return items[index];" 0)
    (nested-zero "const i = 1; const index = i * 0 + 1; return items[index];" 1)
    (typed-alias-zero "const i: Idx = 1; const index = i * 0; return items[index];" 0)
    (nested-alias-zero "const i: Nested = 1; const index = i * 0; return items[index];" 0)
    (alias-wide-right "const i = 2; const one: Idx = 1; return items[i - one];" 1)))

(let* ([forged-ir (checked-narrow-index ir)] [node (car (nodes forged-ir))])
  (parameterize ([current-rust-vector-index-facts (make-rust-vector-index-facts forged-ir)])
    (observe-rust-vector-index! (source node) 0)
    (check 'checked-downcast-not-admitted (not (rust-vector-index node)))))

(for-each
  (lambda (transform label)
    (let* ([forged (transform ir)] [node (car (nodes forged))])
      (parameterize ([current-rust-vector-index-facts (make-rust-vector-index-facts forged)])
        (observe-rust-vector-index! (source node) 1)
        (check label (not (rust-vector-index node))))))
  (list field-arithmetic-index unguarded-subtraction-index)
  '(field-arithmetic-not-admitted unguarded-subtraction-not-admitted))

(let* ([runtime-zero-ir (analyze (write-source "runtime-zero"
         "export pure circuit choose(items: Vector<3, Field>, i: Uint<8>): Field { const index = i * 0; return items[index]; }"))]
       [runtime-zero-node (car (nodes runtime-zero-ir))])
  (parameterize ([current-rust-vector-index-facts (make-rust-vector-index-facts runtime-zero-ir)])
    (observe-rust-vector-index! (source runtime-zero-node) 0)
    (check 'inferred-zero-does-not-admit-runtime-formal (not (rust-vector-index runtime-zero-node)))))

(let* ([runtime-ir (analyze (write-source "runtime-compound-zero"
         "export pure circuit choose(items: Vector<3, Field>, i: Uint<8>): Field { const index = i * 0 + 1; return items[index - 1]; }"))]
       [node (car (nodes runtime-ir))])
  (parameterize ([current-rust-vector-index-facts (make-rust-vector-index-facts runtime-ir)])
    (observe-rust-vector-index! (source node) 0)
    (check 'compound-inferred-zero-still-rejects-runtime-input (not (rust-vector-index node)))))

(printf "~d Rust Vector index occurrence controls passed\n" checks)
