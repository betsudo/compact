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

#!chezscheme

(library (ledger-version)
  (export ledger-version-strings)
  (import (except (chezscheme) errorf))
  (import (utils) (json))

  (define ledger-version-strings
    (let-syntax ([a (lambda (x)
                      ;; Grep flake.nix for an end of line comment matching `# key`.  Allow only one.
                      (define (grep-for key)
                        (let-values ([(stdout stderr) (shell (format "grep 'url = .* # ~a *$' flake.nix | sed -e 's:.*midnight-ledger/\\(.*\\)\";.*:\\1:'" key))])
                          (assertf (string=? stderr "")
                            "grep/sed pipeline produced nonempty stderr for key ~a:\n ~a"
                            key stderr)
                          (let ([ip (open-input-string stdout)])
                            (let ([version-string (get-line ip)])
                              (assertf (not (eof-object? version-string))
                                       "grep/sed pipeline failed to find a match for key ~a" key)
                              (assertf (not (string=? version-string ""))
                                       "grep/sed pipline found an empty vesion string for key ~a" key)
                              (assertf (eof-object? (get-line ip))
                                       "grep/sed pipeline found more than one match for key ~a" key)
                              version-string))))
                      (define (zkir-v3-version)
                        (let ([file "test-center/package.json"])
                          (#%$require-include file)
                          (let* ([manifest (read-json-file file)]
                                 [deps (cdr (assoc "dependencies" manifest))])
                            (cdr (assoc "@midnightntwrk/zkir-v3" deps)))))

                      #`'#,(list (cons "zkir-v2" (grep-for "zkir-v2"))
                                 (cons "zkir-v3" (zkir-v3-version))))])
      a)))
