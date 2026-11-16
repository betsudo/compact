#! /usr/bin/env -S scheme --compile-imported-libraries --program
#!chezscheme

;;; This file is part of Compact.
;;; Copyright (C) 2026 Midnight Foundation
;;; SPDX-License-Identifier: Apache-2.0
;;; Licensed under the Apache License, Version 2.0 (the "License");
;;; you may not use this file except in compliance with the License.

;; Internal generated-text probe only. Runtime 0.18 cannot execute this code:
;; the pinned CallContext, call trace and module metadata are still separate work.
(import (except (chezscheme) errorf) (utils) (passes) (config-params))

(define checks 0)
(define (check name good?)
  (unless good? (error 'cross-contract-print "failed ~s" name))
  (set! checks (+ checks 1)))
(define (read-bytes path)
  (call-with-port (open-file-input-port path) get-bytevector-all))
(define (read-text path)
  (call-with-port (open-input-file path) get-string-all))
(define (digest bytes)
  (let-values ([(out err) (shell "sha256sum -b -z -- && printf ok" bytes)])
    (check 'sha256sum-output
      (and (string=? err "") (= (string-length out) 70)
           (string=? (substring out 64 70) " *-\x0;ok")))
    (substring out 0 64)))
(define (copy-pinned source target expected)
  (let ([bytes (read-bytes source)])
    (check source (string=? (digest bytes) expected))
    (call-with-port (open-file-output-port target (file-options no-fail))
      (lambda (port) (put-bytevector port bytes)))))
(define (contains? value wanted)
  (let ([limit (- (string-length value) (string-length wanted))])
    (let loop ([i 0])
      (and (<= i limit)
           (or (string=? (substring value i (+ i (string-length wanted))) wanted)
               (loop (+ i 1)))))))

(check 'one-output-directory (= (length (command-line)) 2))
(define target (cadr (command-line)))
(mkdir target)
(define callee-source (format "~a/Callee.compact" target))
(define caller-source (format "~a/Caller.compact" target))
(copy-pinned "test-center/composable/EntryPoint/Callee.compact" callee-source
  "89263a08b98928b7b9c4b750588c6bc2226784ceaf62a660511042c2126a51c7")
(copy-pinned "test-center/composable/EntryPoint/Caller.compact" caller-source
  "25a4e30241457760d4c0ec5d317fa283ace17a7b91dd67928a13a6ed390b6df6")
(check 'pinned-entrypoint-test-source
  (string=? (digest (read-bytes "test-center/ts/composable/entry-point.ts"))
    "3b3e8995988e50121749ba5c59a593da326ec58786590e5a688d96ec7740785d"))

(parameterize ([feature-zkir-v3 #t] [compile-contract-statements #t]
               [emit-verifier-key-metadata #t] [skip-zk #t])
  (generate-everything callee-source (format "~a/Callee" target))
  (generate-everything caller-source (format "~a/Caller" target)))

(define caller (read-text (format "~a/Caller/contract/index.js" target)))
(define callee (read-text (format "~a/Callee/contract/index.js" target)))
(define caller-types (read-text (format "~a/Caller/contract/index.d.ts" target)))
;; These are exact generated-code fragments from pinned compiler/test.ss and
;; print-typescript.ss, scoped to the internal test flag.
(check 'awaited-call (contains? caller "await __compactRuntime.crossContractCall({"))
(check 'combined-guards-null-prototype-vk
  (contains? caller "export const expectedVk = {\n  __proto__: null,\n};"))
(check 'combined-guards-complete-called-interface
  (contains? caller "'Callee': {'transfer':"))
(check 'called-interface-retains-unused-decoy
  (contains? caller "'decoy': {argumentTypes:"))
(check 'caller-interface (contains? caller "declaration: declaredInterfaces['Callee']"))
(check 'called-entry-point (contains? caller "calleeCircuitId: 'transfer'"))
(check 'sibling-entry-point-not-substituted (not (contains? caller "calleeCircuitId: 'decoy'")))
(check 'caller-context-copy (contains? caller "const context = __compactRuntime.copyCircuitContext(contextOrig_0);"))
(check 'caller-finalization (contains? caller "__compactRuntime.finalizeCallProofData(context, partialProofData);"))
(check 'callee-finalization (contains? callee "__compactRuntime.finalizeCallProofData(context, partialProofData);"))
(check 'caller-call-context (contains? caller "context.callContext.currentQueryContext.state"))
(check 'callee-call-context (contains? callee "context.callContext.currentQueryContext.state"))
(check 'returned-call-gas (contains? caller "gasCost: context.callContext.currentGasCost"))
(check 'async-caller (contains? caller "async _pay_0(context, partialProofData"))
(check 'async-wrapper (contains? caller "pay: async (...args_1) => {"))
(check 'awaited-caller (contains? caller "await this._pay_0(context, partialProofData"))
(check 'async-declaration (contains? caller-types "Promise<__compactRuntime.CircuitResults<PS, bigint>>"))
;; The sibling has the same shape and is declared, yet the emitted call site
;; names transfer. Proof-level refusal of a substituted call is still pending.
(check 'sibling-exists (contains? (read-text callee-source) "export circuit decoy"))
(printf "internal cross-contract print: ~d checks passed; generated invocation refused.\n" checks)
