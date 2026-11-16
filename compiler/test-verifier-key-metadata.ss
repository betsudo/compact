#! /usr/bin/env -S scheme --compile-imported-libraries --program
#!chezscheme

;;; This file is part of Compact.
;;; Copyright (C) 2026 Midnight Foundation
;;; SPDX-License-Identifier: Apache-2.0
;;; Licensed under the Apache License, Version 2.0 (the "License");
;;; you may not use this file except in compliance with the License.

;; Internal metadata probe only. Generated cross-contract calls remain refused.
(import (except (chezscheme) errorf) (utils) (passes) (config-params))

(define checks 0)
(define (check name good?)
  (unless good? (error 'verifier-key-metadata "failed ~s" name))
  (set! checks (+ checks 1)))
(define (read-bytes path)
  (call-with-port (open-file-input-port path) get-bytevector-all))
(define (read-text path)
  (call-with-port (open-input-file path) get-string-all))
(define (physical-sha256 path)
  (let-values ([(out err) (shell "sha256sum -b -z --" (read-bytes path))])
    (check 'sha256sum-output
      (and (string=? err "") (= (string-length out) 68)
           (string=? (substring out 64 68) " *-\x0;")))
    (substring out 0 64)))
(define (copy-pinned source target expected)
  (let ([bytes (read-bytes source)])
    (check source (string=? (physical-sha256 source) expected))
    (call-with-port (open-file-output-port target (file-options no-fail))
      (lambda (port) (put-bytevector port bytes)))))
(define (contains? value wanted)
  (let ([limit (- (string-length value) (string-length wanted))])
    (let loop ([i 0])
      (and (<= i limit)
           (or (string=? (substring value i (+ i (string-length wanted))) wanted)
               (loop (+ i 1)))))))
(define (failure source output v3? metadata?)
  (guard (condition [else (format-condition condition)])
    (parameterize ([feature-zkir-v3 v3?]
                   [emit-verifier-key-metadata metadata?]
                   [skip-zk #t])
      (generate-everything source output))
    #f))

(check 'one-output-directory (= (length (command-line)) 2))
(define target (cadr (command-line)))
(mkdir target)
(define callee-source (format "~a/Callee.compact" target))
(define caller-source (format "~a/Caller.compact" target))
(copy-pinned "test-center/composable/EntryPoint/Callee.compact" callee-source
  "89263a08b98928b7b9c4b750588c6bc2226784ceaf62a660511042c2126a51c7")
(copy-pinned "test-center/composable/EntryPoint/Caller.compact" caller-source
  "25a4e30241457760d4c0ec5d317fa283ace17a7b91dd67928a13a6ed390b6df6")
(check 'pinned-entrypoint-runtime-source
  (string=? (physical-sha256 "test-center/ts/composable/entry-point.ts")
    "3b3e8995988e50121749ba5c59a593da326ec58786590e5a688d96ec7740785d"))

;; No public compiler mode gains the new export or admits the call.
(parameterize ([feature-zkir-v3 #t] [emit-verifier-key-metadata #f] [skip-zk #t])
  (generate-everything callee-source (format "~a/Callee" target)))
(check 'ordinary-module-has-no-vk-export
  (not (contains? (read-text (format "~a/Callee/contract/index.js" target))
         "export const expectedVk")))
(for-each
  (lambda (v3?)
    (let ([message (failure caller-source (format "~a/refusal-~a" target v3?) v3? #f)])
      (check (if v3? 'ordinary-v3-call-refused 'ordinary-v2-call-refused)
        (and (string? message)
             (contains? message "cross-contract calls are not yet supported")))))
  '(#t #f))
(check 'internal-metadata-requires-v3
  (let ([message (failure callee-source (format "~a/refusal-metadata-v2" target) #f #t)])
    (and (string? message) (contains? message "requires ZKIR v3"))))

(parameterize ([feature-zkir-v3 #t] [emit-verifier-key-metadata #t] [skip-zk #t])
  (generate-everything callee-source (format "~a/skip" target)))
(define skip-js (read-text (format "~a/skip/contract/index.js" target)))
(define skip-dts (read-text (format "~a/skip/contract/index.d.ts" target)))
(check 'skip-zk-empty-vk-table
  (contains? skip-js "export const expectedVk = {\n  __proto__: null,\n};"))
(check 'skip-zk-declaration
  (contains? skip-dts "export declare const expectedVk: Record<string, string>;"))
(check 'existing-signatures-still-null-prototype
  (contains? skip-js "export const circuitSignatures = {\n  __proto__: null,"))
(check 'existing-interfaces-still-null-prototype
  (contains? skip-js "export const declaredInterfaces = { __proto__: null };"))

(parameterize ([feature-zkir-v3 #t] [emit-verifier-key-metadata #t] [skip-zk #f])
  (generate-everything callee-source (format "~a/keys" target)))
(define key-js (read-text (format "~a/keys/contract/index.js" target)))
(for-each
  (lambda (name)
    (let* ([key (format "~a/keys/keys/~a.verifier" target name)]
           [digest (physical-sha256 key)])
      (check (string->symbol (format "physical-~a" name)) (file-exists? key))
      (check (string->symbol (format "embedded-~a" name))
        (contains? key-js (format "'~a': '~a'" name digest)))))
  '(transfer decoy))
(check 'missing-key-refused
  (guard (condition [else #t])
    (collect-verifier-key-hashes (format "~a/keys" target) '(transfer missing) #f)
    #f))
(check 'explicit-skip-ignores-existing-keys
  (null? (collect-verifier-key-hashes (format "~a/keys" target) '(transfer decoy) #t)))

;; An altered file changes the digest; a name or ZKIR source is not a key.
(define mutation (format "~a/mutated" target))
(mkdir mutation)
(mkdir (format "~a/keys" mutation))
(define mutation-key (format "~a/keys/transfer.verifier" mutation))
(call-with-port (open-file-output-port mutation-key)
  (lambda (port) (put-bytevector port #vu8(0 1 2 3))))
(define before (cdr (car (collect-verifier-key-hashes mutation '(transfer) #f))))
(call-with-port (open-file-output-port mutation-key (file-options replace))
  (lambda (port) (put-bytevector port #vu8(0 1 2 4))))
(define after (cdr (car (collect-verifier-key-hashes mutation '(transfer) #f))))
(check 'changed-physical-byte-changes-hash (not (string=? before after)))
(check 'changed-hash-is-physical (string=? after (physical-sha256 mutation-key)))

;; A computed key is required for this spelling: a quoted literal would set
;; Object.prototype rather than create an own verifier entry.
(define proto-source (format "~a/Proto.compact" target))
(call-with-port (open-output-file proto-source)
  (lambda (port)
    (put-string port
      "import CompactStandardLibrary;\nexport ledger balance: Uint<64>;\nconstructor(initial: Uint<64>) { balance = disclose(initial); }\nexport circuit __proto__(amount: Uint<64>): Uint<64> { balance = disclose(balance - amount) as Uint<64>; return balance; }\n")))
(parameterize ([feature-zkir-v3 #t] [emit-verifier-key-metadata #t] [skip-zk #f])
  (generate-everything proto-source (format "~a/Proto" target)))
(define proto-js (read-text (format "~a/Proto/contract/index.js" target)))
(define proto-digest (physical-sha256 (format "~a/Proto/keys/__proto__.verifier" target)))
(check 'prototype-name-emitted-as-own-key
  (contains? proto-js (format "['__proto__']: '~a'" proto-digest)))
(check 'prototype-table-has-null-prototype
  (contains? proto-js "export const expectedVk = {\n  __proto__: null,"))
(check 'missing-prototype-key-refuses
  (guard (condition [else #t])
    (collect-verifier-key-hashes (format "~a/keys" target) '(__proto__) #f)
    #f))

(define enum-source (format "~a/EnumCollision.compact" target))
(call-with-port (open-output-file enum-source)
  (lambda (port)
    (put-string port "import CompactStandardLibrary;\nexport enum expectedVk { A }\nexport circuit get(): Field { return 1; }\n")))
(check 'exported-enum-vk-name-refused
  (let ([message (failure enum-source (format "~a/enum-collision" target) #t #t)])
    (and (string? message)
         (contains? message "exported enum name expectedVk conflicts"))))
(printf "grouped verifier metadata: ~d checks passed; cross-contract calls refused.\n" checks)
