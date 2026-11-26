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

;; Complement compiler/test.ss with well-typed boundary controls. Actual test
;; counts/diagnostics are observations after execution, not Source credit.
(define checks 0)
(define (check name good?)
  (unless good? (error 'rust-set-unit-returns "failed ~s" name))
  (set! checks (+ checks 1)))
(define (get object key)
  (cdr (or (assoc key object) (error 'rust-set-unit-returns "missing ~s" key))))
(define (bytes path) (call-with-port (open-file-input-port path) get-bytevector-all))
(define (contains? text wanted)
  (let ([limit (- (string-length text) (string-length wanted))])
    (let loop ([i 0])
      (and (<= i limit)
           (or (string=? (substring text i (+ i (string-length wanted))) wanted)
               (loop (+ i 1)))))))
(define (compile-case source output rust?)
  (parameterize ([feature-zkir-v3 #t] [ts-runtime-profile 'released-0.21.0]
                 [emit-rust-ir rust?] [skip-zk #t])
    (generate-everything source output)))
(check 'one-fresh-output-directory (= (length (command-line)) 2))
(define target (cadr (command-line)))
(mkdir target)
(define source "examples/adt/exports/set_field.compact")
(define pinned "tools/compact-rust-backend/fixtures/upstream-672/export-set-field")
(check 'intact-six-export-source
  (equal? (bytes source) (bytes (format "~a/original-set-field.compact" pinned))))
(define pinned-test-source (format "~a/original-set-field-test.compact" pinned))
(check 'authenticated-pinned-ten-assert-test-source
  (and (= (bytevector-length (bytes pinned-test-source)) 1459)
       ;; This relative fixture path is a fixed ASCII repository-owned literal.
       ;; Follow manifest-passes' real sha256sum protocol; no current-example alias.
       (let-values ([(stdout stderr)
                      (shell (format "exec sha256sum -b -z -- ~a" pinned-test-source))])
         (and (string=? stderr "") (>= (string-length stdout) 64)
              (string=? (substring stdout 0 64)
                "07a6b0aae4ce176b55fd33e7004856461dfe0922cc53c799f11bd1e3fb76e673")))))
(compile-case source (format "~a/original" target) #t)
(define ir (read-json-file (format "~a/original/contract/compact-rust-ir.json" target)))
(define circuits (vector->list (get ir "stateful_circuits")))
(check 'six-exports-in-original-order
  (equal? (map (lambda (c) (get c "name")) circuits)
          '("reset_to_default" "insert" "is_empty" "member" "remove" "size")))
(define (mutation-object-keys? object keys)
  (and (list? object) (for-all pair? object)
       (equal? (list-sort string<? (map car object)) (list-sort string<? keys))))
(define (mutation-field-parameters? parameters parameter)
  (and (vector? parameters)
       (if parameter
           (and (= (vector-length parameters) 1)
                (let ([p (vector-ref parameters 0)])
                  (and (mutation-object-keys? p '("name" "ty"))
                       (equal? (get p "name") parameter)
                       (mutation-object-keys? (get p "ty") '("kind"))
                       (equal? (get (get p "ty") "kind") "field"))))
           (= (vector-length parameters) 0))))
(define (mutation-action? action kind field index parameter)
  (and (mutation-object-keys? action
           (if parameter '("kind" "field" "index" "value") '("kind" "field" "index")))
       (equal? (get action "kind") kind)
       (equal? (get action "field") field)
       (integer? (get action "index")) (= (get action "index") index)
       (or (not parameter)
           (let ([value (get action "value")])
             (and (mutation-object-keys? value '("kind" "name"))
                  (equal? (get value "kind") "parameter")
                  (equal? (get value "name") parameter))))))
(define (one-mutation c field index parameter kind)
  (let* ([returned (get c "return_value")]
         [body (get returned "body")]
         [actions (get body "actions")])
    (and (mutation-field-parameters? (get c "parameters") parameter)
         (equal? (get c "result") '(("kind" . "unit")))
         (equal? (get c "actions") '#())
         (string=? (get returned "kind") "effectful")
         (string=? (get body "kind") "sequence")
         (= (vector-length actions) 1)
         (mutation-action? (vector-ref actions 0) kind field index parameter)
         (equal? (get body "result")
           '(("kind" . "value") ("value" . (("kind" . "unit"))))))))
(for-each
  (lambda (name kind parameter)
    (check 'original-mutation-effect-once-then-unit
      (one-mutation (find (lambda (c) (string=? (get c "name") name)) circuits)
                    "c" 0 parameter kind)))
  '("reset_to_default" "insert" "remove") '("set_reset" "set_insert" "set_remove")
  '(#f "value" "value"))
;; The original test's assertions are retained but not executed by generation.
(compile-case pinned-test-source (format "~a/ten-assert-test" target) #t)
(check 'intact-original-test-generates #t)

(define (write-source name text)
  (let ([path (format "~a/~a.compact" target name)])
    (call-with-output-file path (lambda (p) (display text p))) path))
(define (accept name text export field index parameter)
  (let ([path (write-source name text)])
    (compile-case path (format "~a/~a-rust" target name) #t)
    (let* ([ir (read-json-file (format "~a/~a-rust/contract/compact-rust-ir.json" target name))]
           [c (find (lambda (c) (string=? (get c "name") export))
                    (vector->list (get ir "stateful_circuits")))])
      (check 'semantic-admission-not-fixture-spelling (one-mutation c field index parameter "set_remove")))))
(accept "renamed"
  "import CompactStandardLibrary; export ledger values: Set<Field>; export circuit drop(k: Field): [] { return values.remove(disclose(k)); }"
  "drop" "values" 0 "k")
(accept "root-index-fourteen"
  (string-append "import CompactStandardLibrary; "
    (apply string-append (map (lambda (i) (format "export ledger p~a: Counter; " i)) (iota 14)))
    "export ledger values: Set<Field>; export circuit drop(k: Field): [] { return values.remove(disclose(k)); }")
  "drop" "values" 14 "k")

;; TS generation must succeed first. A failure in this control establishes
;; no Rust boundary; the test stops. Only then require an actual Rust-owned
;; diagnostic with the stable message observed in the first characterization.
;; Full diagnostics are still saved; these are not runtime/crypto refusals.
(define (refuse name text expected)
  (let ([path (write-source name text)])
    (compile-case path (format "~a/~a-ts" target name) #f)
    (check 'well-typed-ts-positive-before-rust-refusal #t)
    (let ([diagnostic
            (guard (c [else (format-condition c)])
              (compile-case path (format "~a/~a-rust" target name) #t) #f)])
      (call-with-output-file (format "~a/~a-refusal.txt" target name)
        (lambda (p) (write diagnostic p) (newline p)))
      (check 'observed-owner-specific-rust-refusal
        (and (string? diagnostic) (contains? diagnostic expected))))))
(refuse "other-set-element"
  "import CompactStandardLibrary; export ledger c: Set<Boolean>; export circuit remove(v: Boolean): [] { return c.remove(disclose(v)); }"
  "Rust backend does not yet support this ledger return operation")
(refuse "witness-argument"
  "import CompactStandardLibrary; witness value(): Field; export ledger c: Set<Field>; export circuit remove(): [] { return c.remove(disclose(value())); }"
  "Rust backend does not yet support this nested ledger query: Set.remove")
(refuse "effectful-helper-argument"
  "import CompactStandardLibrary; export ledger c: Set<Field>; circuit value(): Field { c.insert(9 as Field); return 1; } export circuit remove(): [] { return c.remove(disclose(value())); }"
  "Rust backend does not yet support this nested ledger query: Set.remove")
(refuse "ledger-read-argument"
  "import CompactStandardLibrary; export ledger c: Set<Field>; export ledger q: Counter; export circuit remove(): [] { return c.remove(disclose(q.read() as Field)); }"
  "Rust backend does not yet support this nested ledger query: Set.remove")
;; EARLIER_UNSUPPORTED_NESTED_ADT_TYPE: no root path-guard refusal credit.
(refuse "nested-path"
  "import CompactStandardLibrary; export ledger c: Map<Field, Set<Field>>; export circuit remove(k: Field): [] { return c.lookup(disclose(k)).remove(disclose(k)); }"
  "Rust backend does not yet support this ledger ADT type")
(refuse "conditional-removal-return"
  "import CompactStandardLibrary; export ledger c: Set<Field>; export circuit remove(flag: Boolean, v: Field): [] { if (disclose(flag)) { return c.remove(disclose(v)); } else { return c.remove(0 as Field); } }"
  "Rust backend does not yet support this nested ledger query: Set.remove")
(refuse "effectful-prefix"
  "import CompactStandardLibrary; export ledger c: Set<Field>; export circuit remove(v: Field): [] { c.insert(8 as Field); return c.remove(disclose(v)); }"
  "Rust backend does not yet support this ledger return operation")
(printf "Rust terminal Set Unit returns: ~d checks passed.\n" checks)
