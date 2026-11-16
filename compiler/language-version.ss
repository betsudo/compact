#!chezscheme

;;; This file is part of Compact.
;;; Copyright (C) 2025 Midnight Foundation
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

(library (language-version)
  (export language-version-string check-language-version)
  (import (chezscheme) (config-params) (version))

  ;; The v2 compiler remains the Ledger 8 language profile. The explicit
  ;; ZKIR v3 target selects the language version shipped by Compact 0.36.0.
  (define ledger8-language-version (make-version 'language 0 23 105))
  ;; Host version readers select the single literal make-version form above.
  (define ledger9-language-major 0)
  (define ledger9-language-version
    (make-version 'language ledger9-language-major 28 0))

  (define (selected-language-version)
    (if (feature-zkir-v3) ledger9-language-version ledger8-language-version))

  (define-syntax language-version-string
    (identifier-syntax (make-version-string (selected-language-version))))

  (define ledger8-check-language-version (make-version-checker ledger8-language-version))
  (define ledger9-check-language-version (make-version-checker ledger9-language-version))

  (define (check-language-version src ve)
    ((if (feature-zkir-v3)
         ledger9-check-language-version
         ledger8-check-language-version)
     src ve))
)
