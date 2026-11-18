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

;; Source-statement qualification only. The pinned EntryPoint runtime test,
;; proofs, generated invocation and funded transaction are separate gates.
(import (except (chezscheme) errorf)
        (utils)
        (langs)
        (passes)
        (config-params))

(define checks 0)
(define (check name good?)
  (unless good? (error 'entrypoint-statement "failed ~s" name))
  (set! checks (+ checks 1)))

(define (read-bytes source)
  (call-with-port (open-file-input-port source) get-bytevector-all))

(define (digest bytes)
  (let-values ([(out err) (shell "sha256sum -b -z -- && printf ok" bytes)])
    (check 'sha256sum-output
      (and (string=? err "")
           (= (string-length out) 70)
           (string=? (substring out 64 70) " *-\x0;ok")))
    (substring out 0 64)))

(define (copy-pinned source target expected)
  (let ([bytes (read-bytes source)])
    (check source (string=? (digest bytes) expected))
    (call-with-port (open-file-output-port target (file-options no-fail))
      (lambda (port) (put-bytevector port bytes)))))

(define (collect-nodes tree wanted?)
  (let loop ([node tree])
    (append (if (wanted? node) (list node) '())
            (if (pair? node)
                (append (loop (car node)) (loop (cdr node)))
                '()))))

(define (tagged? tag node)
  (and (pair? node) (eq? (car node) tag)))

(define (contains? value tree)
  (not (null? (collect-nodes tree (lambda (node) (equal? node value))))))

(define (only-one name items)
  (check name (= (length items) 1))
  (car items))

(define (statement-op statement) (list-ref statement 3))
(define (statement-outputs statement)
  (let ([outputs (list-ref statement 2)])
    (if (list? outputs) outputs (list outputs))))
(define (call-target statement) (list-ref (statement-op statement) 1))
(define (call-args statement) (list-tail (statement-op statement) 2))
(define (ledger-args statement) (list-tail (statement-op statement) 4))

(check 'one-temporary-directory (= (length (command-line)) 2))
(define target (cadr (command-line)))
(mkdir target)
(define callee-source (format "~a/Callee.compact" target))
(define caller-source (format "~a/Caller.compact" target))
(copy-pinned "test-center/composable/EntryPoint/Callee.compact" callee-source
  "89263a08b98928b7b9c4b750588c6bc2226784ceaf62a660511042c2126a51c7")
(copy-pinned "test-center/composable/EntryPoint/Caller.compact" caller-source
  "25a4e30241457760d4c0ec5d317fa283ace17a7b91dd67928a13a6ed390b6df6")
(check 'pinned-runtime-refusal-source
  (string=? (digest (read-bytes "test-center/ts/composable/entry-point.ts"))
    "3b3e8995988e50121749ba5c59a593da326ec58786590e5a688d96ec7740785d"))

(define flattened #f)
(parameterize ([feature-zkir-v3 #t]
               [compile-contract-statements #t]
               [skip-zk #t])
  ;; The intact callee supplies contract-info.json for the intact caller.
  ;; No key generation, proof or application occurs in this source gate.
  (generate-everything callee-source (format "~a/Callee" target))
  (generate-everything caller-source (format "~a/Caller" target)
    'flatten-datatypes
    (lambda (name unparse pretty-formats ir)
      (when (eq? name 'flatten-datatypes)
        (set! flattened (unparse ir))))))

(check 'caller-flattened flattened)
(define natives
  (collect-nodes flattened (lambda (node) (tagged? 'native node))))
(define transient-native (only-one 'single-transient-commit-native natives))
(define transient-name (list-ref transient-native 1))
(define statements
  (collect-nodes flattened (lambda (node) (tagged? '= node))))
(define transient-call
  (only-one 'single-transient-commit-call
    (filter (lambda (node)
              (let ([op (statement-op node)])
                (and (tagged? 'call op)
                     (eq? (call-target node) transient-name))))
            statements)))
(define claim
  (only-one 'single-contract-claim
    (filter (lambda (node)
              (let ([op (statement-op node)])
                (and (tagged? 'public-ledger op)
                     (contains? 'claimContractCall (list-ref op 3)))))
            statements)))

;; The claim uses the exact commitment produced from caller arguments, the
;; callee result, and its opening. It also uses the two limbs read from the
;; caller's `callee` ledger field, not a standalone address supplied by host.
(define claim-args (ledger-args claim))
(check 'one-address-two-hash-limbs-and-commitment (= (length claim-args) 5))
(define receiver (list-head claim-args 2))
(define entry-point (list-head (list-tail claim-args 2) 2))
(define commitment (list-ref claim-args 4))
(check 'commitment-output (= (length (statement-outputs transient-call)) 1))
(check 'claim-binds-commitment (eq? commitment (car (statement-outputs transient-call))))
(define address-reads
  (filter (lambda (node)
            (let ([op (statement-op node)])
              (and (tagged? 'public-ledger op)
                   (contains? 'read (list-ref op 3))
                   (= (length (statement-outputs node)) 2))))
          statements))
(check 'claim-binds-observed-callee-address
  (exists (lambda (node) (equal? receiver (statement-outputs node))) address-reads))

;; Independent SHA-256 oracle over the Ledger 9 domain and UTF-8 name.
;; These are the high 1-byte and low 31-byte limbs of transfer's hash,
;; in the compiler's flattened order.
(define transfer-limbs
  '(158 13895292137595814349528795736747648284072927302011983156518773832301390552))
(define decoy-limbs
  '(177 116185992629533854605212197648327734421581847960737194425877946469672195153))
(check 'fixed-transfer-entry-point (equal? entry-point transfer-limbs))
(check 'same-shape-decoy-is-distinct (not (equal? entry-point decoy-limbs)))
(define (bound-claim? args)
  (and (= (length args) 5)
       (exists (lambda (node) (equal? (list-head args 2) (statement-outputs node)))
               address-reads)
       (equal? (list-head (list-tail args 2) 2) transfer-limbs)
       (eq? (list-ref args 4) (car (statement-outputs transient-call)))))
(check 'honest-statement-bound (bound-claim? claim-args))

(define callee-result-call
  (only-one 'single-callee-result-witness
    (filter (lambda (node)
              (let ([op (statement-op node)])
                (and (tagged? 'call op)
                     (not (eq? (call-target node) transient-name))
                     (= (length (statement-outputs node)) 2))))
            statements)))
(define witness-args (call-args callee-result-call))
(define witness-outputs (statement-outputs callee-result-call))
(define commitment-args (call-args transient-call))
(check 'one-original-call-argument (= (length witness-args) 1))
(check 'argument-result-opening-commitment (= (length commitment-args) 3))
(check 'original-argument-bound (eq? (car witness-args) (car commitment-args)))
(check 'callee-result-bound (eq? (car witness-outputs) (cadr commitment-args)))
(check 'commitment-opening-bound (eq? (cadr witness-outputs) (caddr commitment-args)))

;; Mutation controls make the binding assertions sensitive to both attack
;; classes. Runtime/proof refusal is still required in the later call lane.
(check 'wrong-address-control
  (not (bound-claim? (cons 'wrong-address (cdr claim-args)))))
(check 'sibling-entry-point-control
  (not (bound-claim? (append receiver decoy-limbs (list commitment)))))

(define (has-substring? value wanted)
  (let ([limit (- (string-length value) (string-length wanted))])
    (let loop ([i 0])
      (and (<= i limit)
           (or (string=? (substring value i (+ i (string-length wanted))) wanted)
               (loop (+ i 1)))))))

;; Public compiler modes remain fail closed until generated invocation exists.
(for-each
  (lambda (v3?)
    (let ([refusal
            (guard (condition [else (format-condition condition)])
              (parameterize ([feature-zkir-v3 v3?]
                             [compile-contract-statements #f]
                             [skip-zk #t])
                (generate-everything caller-source
                  (format "~a/ordinary-~a" target (if v3? "v3" "v2"))
                  (if v3? 'flatten-datatypes 'print-zkir) #f))
              #f)])
      (check (if v3? 'ordinary-v3-refuses 'ordinary-v2-refuses)
        (and (string? refusal)
             (has-substring? refusal "cross-contract calls are not yet supported")))
      (printf "ordinary ~a refusal: ~a\n" (if v3? 'v3 'v2) refusal)))
  '(#t #f))
(define internal-printer-result
  (guard (condition [else (format-condition condition)])
    (parameterize ([feature-zkir-v3 #t]
                   [compile-contract-statements #t]
                   [skip-zk #t])
      (generate-everything caller-source (format "~a/internal-print" target)))
    #t))
(check 'internal-statement-printer-reaches-guarded-call
  (and (eq? internal-printer-result #t)
       (has-substring?
         (call-with-input-file (format "~a/internal-print/contract/index.js" target)
           get-string-all)
         "await __compactRuntime.crossContractCall({")))
(printf "EntryPoint source statement: ~d checks passed; proof/runtime gates pending.\n" checks)
