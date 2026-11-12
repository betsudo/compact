#!chezscheme

;;; This file is part of Compact.
;;; Copyright (C) 2026 Midnight Foundation
;;; SPDX-License-Identifier: Apache-2.0
;;; Licensed under the Apache License, Version 2.0 (the "License");
;;; you may not use this file except in compliance with the License.
;;; You may obtain a copy of the License at
;;;
;;; 	http://www.apache.org/licenses/LICENSE-2.0
;;;
;;; Unless required by applicable law or agreed to in writing, software
;;; distributed under the License is distributed on an "AS IS" BASIS,
;;; WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
;;; See the License for the specific language governing permissions and
;;; limitations under the License.

(library (contract-entry-point)
  (export contract-entry-point-hash)
  (import (except (chezscheme) errorf) (utils))

  (define (hex-digit? c)
    (or (char<=? #\0 c #\9)
        (char<=? #\a c #\f)
        (char<=? #\A c #\F)))

  (define (hash-preimage preimage)
    ;; sha256sum also produces the contract manifest's digests. The marker
    ;; follows its NUL-terminated stdin record only after a successful exit.
    (let-values ([(stdout stderr) (shell "sha256sum -b -z -- && printf ok" preimage)])
      (unless (string=? stderr "")
        (external-errorf "sha256sum failed while hashing a contract entry point: ~a" stderr))
      (unless (and (= (string-length stdout) 70)
                   (string=? (substring stdout 64 70) " *-\x0;ok")
                   (andmap hex-digit? (string->list (substring stdout 0 64))))
        (external-errorf "invalid sha256sum output while hashing a contract entry point"))
      (let ([digest (make-bytevector 32)])
        (bytevector-uint-set! digest 0 (string->number (substring stdout 0 64) 16)
          (endianness big) 32)
        digest)))

  ;; Ledger 9 EntryPointBuf::ep_hash commits to the UTF-8 name under a fixed
  ;; 32-byte domain. Copy both inputs as bytes so embedded zeroes survive.
  (define (contract-entry-point-hash elt-name)
    (let* ([domain (string->utf8 "midnight:entry-point")]
           [name (string->utf8 (symbol->string elt-name))]
           [preimage (make-bytevector (+ 32 (bytevector-length name)) 0)])
      (bytevector-copy! domain 0 preimage 0 (bytevector-length domain))
      (bytevector-copy! name 0 preimage 32 (bytevector-length name))
      (hash-preimage preimage))))
