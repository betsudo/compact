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

;; Keep the pinned original fixture intact. Source Rust IR admission is only
;; for the released Ledger 9 profile; legacy v2 and v3 remain refused.
(import (except (chezscheme) errorf) (utils) (passes) (config-params))

(define checks 0)
(define (check name good?)
  (unless good? (error 'rust-block-time-profile "failed ~s" name))
  (set! checks (+ checks 1)))
(define (contains? value wanted)
  (let ([limit (- (string-length value) (string-length wanted))])
    (let loop ([index 0])
      (and (<= index limit)
           (or (string=? (substring value index (+ index (string-length wanted))) wanted)
               (loop (+ index 1)))))))
(define (failure thunk)
  (guard (condition [else (format-condition condition)])
    (thunk)
    #f))
(define (digest source)
  (let ([bytes (call-with-port (open-file-input-port source) get-bytevector-all)])
    (let-values ([(out err) (shell "sha256sum -b -z -- && printf ok" bytes)])
      (check 'digest-tool (and (string=? err "") (= (string-length out) 70)))
      (substring out 0 64))))

(check 'one-output-directory (= (length (command-line)) 2))
(define target (cadr (command-line)))
(define source "test-center/compact/block-time.compact")
(check 'pinned-source
  (string=? (digest source) "33ff8fd24ef8d3a8920e4e519bd86d6423b90570110cd55fc6e4afeb2a48d00d"))
(check 'pinned-original-test
  (string=? (digest "test-center/ts/ledger9-original/block-time.ts")
    "fd1fd834b2189420a80f7e688a1a3423442d4b599ea790de1cac75bf31bde818"))
(mkdir target)
(define (qualify-rust-block-time-profile profile v3?)
  (parameterize ([feature-zkir-v3 v3?]
                 [ts-runtime-profile profile]
                 [emit-rust-ir #t]
                 [skip-zk #t])
    (generate-everything source (format "~a/~a" target
      (if v3? (if profile "released" "legacy-v3") "legacy-v2")))))
(for-each
  (lambda (variant)
    (let ([diagnostic (failure (lambda () (qualify-rust-block-time-profile (car variant) (cdr variant))))])
      (check 'legacy-rust-refusal
        (and (string? diagnostic)
             (contains? diagnostic "Rust backend does not yet support Kernel operation blockTime")))))
  (list (cons #f #f) (cons #f #t)))
(qualify-rust-block-time-profile 'released-0.21.0 #t)
(define ir (call-with-input-file (format "~a/released/contract/compact-rust-ir.json" target)
             get-string-all))
(check 'lowered-strict-less-than (contains? ir "\"kind\": \"kernel_block_time_less_than\""))
(check 'lowered-strict-greater-than (contains? ir "\"kind\": \"kernel_block_time_greater_than\""))
(check 'four-original-exports
  (and (contains? ir "\"name\": \"testBlockTimeLt\"")
       (contains? ir "\"name\": \"testBlockTimeLte\"")
       (contains? ir "\"name\": \"testBlockTimeGt\"")
       (contains? ir "\"name\": \"testBlockTimeGte\"")))
(printf "Rust block-time profile source gate: ~d checks passed.\n" checks)
