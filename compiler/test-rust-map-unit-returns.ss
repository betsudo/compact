#! /usr/bin/env -S scheme --compile-imported-libraries --program
#!chezscheme

;;; This file is part of Compact.
;;; Copyright (C) 2026 Midnight Foundation
;;; SPDX-License-Identifier: Apache-2.0
;;; Licensed under the Apache License, Version 2.0 (the "License");
;;; you may not use this file except in compliance with the License.
;;; You may obtain a copy of the License at
;;;
;;; http://www.apache.org/licenses/LICENSE-2.0
;;;
;;; Unless required by applicable law or agreed to in writing, software
;;; distributed under the License is distributed on an "AS IS" BASIS,
;;; WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
;;; See the License for the specific language governing permissions and
;;; limitations under the License.

(import (except (chezscheme) errorf) (utils) (passes) (config-params) (json))

;; This focused gate compiles the intact original eight-export fixture. It
;; checks the frontend effect owner, not proof generation or runtime results.
(define checks 0)
(define (check name good?)
  (unless good? (error 'rust-map-unit-returns "failed ~s" name))
  (set! checks (+ checks 1)))
(define (get object key) (cdr (or (assoc key object) (error 'rust-map-unit-returns "missing ~s" key))))
(define (bytes path) (call-with-port (open-file-input-port path) get-bytevector-all))
(define (contains? value wanted)
  (let ([limit (- (string-length value) (string-length wanted))])
    (let loop ([index 0])
      (and (<= index limit)
           (or (string=? (substring value index (+ index (string-length wanted))) wanted)
               (loop (+ index 1)))))))
(define (compile-case source output rust?)
  (parameterize ([feature-zkir-v3 #t] [ts-runtime-profile 'released-0.21.0]
                 [emit-rust-ir rust?] [skip-zk #t])
    (generate-everything source output)))
(check 'one-output-directory (= (length (command-line)) 2))
(define target (cadr (command-line)))
(mkdir target)
(define source "examples/adt/exports/map_field_field.compact")
(check 'intact-pinned-eight-export-source
  (equal? (bytes source)
          (bytes "tools/compact-rust-backend/fixtures/upstream-672/export-map-field-field/original-map-field-field.compact")))
(compile-case source (format "~a/original" target) #t)
(define ir (read-json-file (format "~a/original/contract/compact-rust-ir.json" target)))
(define circuits (vector->list (get ir "stateful_circuits")))
(check 'eight-exports (= (length circuits) 8))
(for-each
  (lambda (name kind)
    (let* ([c (find (lambda (c) (string=? (get c "name") name)) circuits)]
           [returned (get c "return_value")]
           [body (get returned "body")]
           [actions (get body "actions")])
      (check 'one-ordered-mutation
        (and (equal? (get c "actions") '#())
             (string=? (get returned "kind") "effectful")
             (string=? (get body "kind") "sequence")
             (= (vector-length actions) 1)
             (string=? (get (vector-ref actions 0) "kind") kind)
             (string=? (get (vector-ref actions 0) "field") "c")
             (= (get (vector-ref actions 0) "index") 0)
             (equal? (get body "result")
                     '(("kind" . "value") ("value" . (("kind" . "unit")))))))))
  '("reset_to_default" "insert" "insert_default" "remove")
  '("map_reset" "map_insert" "map_insert_default" "map_remove"))
(define member-circuit (find (lambda (c) (string=? (get c "name") "member")) circuits))
(check 'actual-member-shape
  (and (equal? (get member-circuit "actions") '#())
       (equal? (get member-circuit "result") '(("kind" . "boolean")))
       (let ([read (get member-circuit "return_value")])
         (and (string=? (get read "kind") "map_member")
              (string=? (get read "field") "c") (= (get read "index") 0)
              (equal? (get read "key")
                '(("kind" . "parameter") ("name" . "key")))))))

;; Each supplemental source must first compile through ordinary TypeScript
;; generation, establishing that the later rejection is a Rust boundary.
(define (refuse name body expected)
  (let ([path (format "~a/~a.compact" target name)])
    (call-with-output-file path (lambda (p) (display body p)))
    (compile-case path (format "~a/~a-ts" target name) #f)
    (check 'well-typed-source-before-rust #t)
    (let ([diagnostic
            (guard (c [else (format-condition c)])
              (compile-case path (format "~a/~a-rust" target name) #t)
              #f)])
      (call-with-output-file (format "~a/~a-refusal.txt" target name)
        (lambda (p) (write diagnostic p) (newline p)))
      (unless (and (string? diagnostic) (contains? diagnostic expected))
        (error 'rust-map-unit-returns "unexpected Rust refusal for ~s: ~s" name diagnostic))
      (check 'bounded-rust-refusal #t))))
(refuse "other-key-type"
  "import CompactStandardLibrary; export ledger c: Map<Boolean, Field>; export circuit remove(k: Boolean): [] { return c.remove(disclose(k)); }"
  "Rust backend does not yet support this ledger return operation")
(refuse "witness-key"
  "import CompactStandardLibrary; witness key(): Field; export ledger c: Map<Field, Field>; export circuit remove(): [] { return c.remove(disclose(key())); }"
  "Rust backend does not yet support this nested ledger query: Map.remove")
(refuse "nested-path"
  "import CompactStandardLibrary; export ledger c: Map<Field, Map<Field, Field>>; export circuit remove(k: Field): [] { return c.lookup(disclose(k)).remove(disclose(k)); }"
  "Rust backend requires an array-index ledger path")
(printf "Rust terminal Map Unit returns: ~d checks passed.\n" checks)
