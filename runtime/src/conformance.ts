// This file is part of Compact.
// Copyright (C) 2026 Midnight Foundation
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

/**
 * Whether a dynamically resolved module implements the contract type its caller declared.
 *
 * Key agreement relates a module to the chain; this relates it to what the caller declared. Skip it
 * and a Vault address passed where a Token was expected resolves to the Vault module, whose keys
 * agree with the chain perfectly and whose `transfer` means something else.
 *
 * What it compares is shape, never identity. Two contract types whose circuits agree in name, arity
 * and types are interchangeable here, and so are two structs or `new type`s that share a name and
 * shape.
 *
 * The comparison ports `sametype?`/`circuit-superset?` (analysis-passes/infer-types.ss:131-233) onto
 * the encoded signatures, deviating only where noted.
 */

import { CompactError } from './error.js';
import { CircuitSignatures, InterfaceCircuitDeclaration, InterfaceDescriptor, SignatureType } from './interface-descriptor.js';

// This synchronous runtime has no circuit-context module yet. Circuit IDs are
// external circuit names, as in the pinned upstream CircuitId alias.
type CircuitId = string;

/** Which rule a module failed. Ordered as they are applied, per circuit. */
export type ConformanceCheck =
  /** The declaration names a circuit the module does not implement. */
  | 'Existence'
  /** The implementation is not among the module's provable circuits, so there is no proof to match a call against. */
  | 'Provability'
  /** The two disagree on how many parameters the circuit takes. */
  | 'Arity'
  /** A positional parameter type differs. `argumentIndex` says which. */
  | 'ArgumentType'
  /** The return types differ. */
  | 'ResultType';

/** The first rule a module failed, and where. */
export type ConformanceViolation = {
  readonly circuitId: CircuitId;
  readonly check: ConformanceCheck;
  readonly argumentIndex?: number;
};

/**
 * A type in a module's signatures this build can't read: an unknown constructor, or a known one
 * whose payload is not what that constructor carries. Distinct from a violation, which says
 * something different from what was asked for; this says something we can't read at all.
 */
export type UnreadableSignature = {
  readonly circuitId: CircuitId;
  readonly unreadableTag: string;
  readonly argumentIndex?: number;
};

/** The outcome of checking a module against a contract type. Tagged, so a call site has to handle
 *  all three. */
export type ConformanceResult =
  | { readonly outcome: 'Conformant' }
  | ({ readonly outcome: 'Violation' } & ConformanceViolation)
  | ({ readonly outcome: 'Unreadable' } & UnreadableSignature);

/**
 * Every tag {@link signatureTypesEqual} compares. Mapped over the union, so a new variant fails to
 * compile until it is listed here.
 */
const KNOWN_SIGNATURE_TAGS: { readonly [K in SignatureType['tag']]: true } = {
  Boolean: true,
  Field: true,
  JubjubScalar: true,
  JubjubPoint: true,
  Secp256k1Base: true,
  Secp256k1Scalar: true,
  Secp256k1Point: true,
  Secp256r1Base: true,
  Secp256r1Scalar: true,
  Secp256r1Point: true,
  Curve25519Base: true,
  Curve25519Scalar: true,
  Curve25519Point: true,
  Uint: true,
  Bytes: true,
  Opaque: true,
  Vector: true,
  Tuple: true,
  Enum: true,
  Struct: true,
  Alias: true,
  Contract: true,
};

/** Reported where a signature is not shaped like one at all, so it has no tag to name. */
const MALFORMED = '(malformed)';
const MAX_SIGNATURE_DEPTH = 128;
const MAX_SIGNATURE_NODES = 10_000;
type ValidationBudget = { remaining: number };

// A declared interface comes from the caller's own generated module. Size the work allowance from
// its encoded shape, rather than a Vector's logical length. The fixed extra allowance still bounds
// how much malformed implementation metadata can make us inspect per declared circuit.
const MAX_DECLARATION_COST = Number.MAX_SAFE_INTEGER - MAX_SIGNATURE_NODES;
const boundedAdd = (a: number, b: number): number => Math.min(MAX_DECLARATION_COST, a + b);
const boundedMultiply = (a: number, b: number): number =>
  a === 0 || b === 0 ? 0 : a > MAX_DECLARATION_COST / b ? MAX_DECLARATION_COST : a * b;

const declarationTypeCost = (type: SignatureType, active: Set<object>, depth = 0, expandVectors = false): number => {
  if (depth > MAX_SIGNATURE_DEPTH || active.has(type)) return 0;
  active.add(type);
  try {
    switch (type.tag) {
      case 'Enum':
        return boundedAdd(1, type.elements.length);
      case 'Alias':
        return boundedAdd(1, declarationTypeCost(type.type, active, depth + 1, expandVectors));
      case 'Vector':
        return boundedAdd(
          1,
          expandVectors
            ? boundedMultiply(type.length, declarationTypeCost(type.type, active, depth + 1, true))
            : declarationTypeCost(type.type, active, depth + 1),
        );
      case 'Tuple':
        return type.types.reduce(
          (cost, element) => boundedAdd(cost, declarationTypeCost(element, active, depth + 1, expandVectors)),
          1,
        );
      case 'Struct':
        return type.elements.reduce(
          (cost, element) => boundedAdd(cost, declarationTypeCost(element.type, active, depth + 1, expandVectors)),
          1,
        );
      case 'Contract':
        return Object.values(type.circuits).reduce(
          (cost, circuit) => boundedAdd(cost, declarationCircuitCost(circuit, active, depth + 1, expandVectors)),
          1,
        );
      default:
        return 1;
    }
  } finally {
    active.delete(type);
  }
};

const declarationCircuitCost = (
  circuit: InterfaceCircuitDeclaration,
  active: Set<object>,
  depth = 0,
  expandVectors = false,
): number =>
  boundedAdd(
    circuit.argumentTypes.reduce(
      (cost, type) => boundedAdd(cost, declarationTypeCost(type, active, depth + 1, expandVectors)),
      1,
    ),
    declarationTypeCost(circuit.resultType, active, depth + 1, expandVectors),
  );

// The declaration is trusted compiler output. Each argument gets its own encoded-shape allowance,
// so an unrelated argument or circuit cannot borrow a large declaration's validation budget.
const validationBudget = (declared: SignatureType, expandVectors = false): ValidationBudget => ({
  remaining: Math.min(
    Number.MAX_SAFE_INTEGER,
    MAX_SIGNATURE_NODES + declarationTypeCost(declared, new Set<object>(), 0, expandVectors),
  ),
});

// Snapshot only a bounded amount of the untrusted signature graph before comparing it. A Proxy
// can mutate a previously memoized node from a property read; structuredClone refuses Proxies,
// and comparison against its detached result never reads the original graph again. Preflight
// checks own descriptors first so ordinary accessors cannot run during cloning. Physical DAG
// nodes are visited once, while array lengths and own properties consume the allowance.
const MAX_SNAPSHOT_ENTRIES = 100_000;
const MAX_SNAPSHOT_BYTES = 8 * 1024 * 1024;
const MAX_SNAPSHOT_CONTAINER_DEPTH = 1_024;
const snapshotSignature = (
  declared: SignatureType,
  implemented: SignatureType,
): { value?: SignatureType; unreadableTag?: string } => {
  const allowance = Math.min(
    MAX_SNAPSHOT_ENTRIES,
    MAX_SIGNATURE_NODES + boundedMultiply(3, declarationTypeCost(declared, new Set<object>(), 0, true)),
  );
  const budget: ValidationBudget = { remaining: allowance };
  const bytes: ValidationBudget = { remaining: MAX_SNAPSHOT_BYTES };
  const active = new Set<object>();
  const seen = new Set<object>();
  const inspect = (value: unknown, typeDepth: number, containerDepth: number, parentTag: string): string | undefined => {
    if (typeof value === 'bigint') return parentTag;
    if (typeof value === 'string') {
      bytes.remaining -= value.length * 2;
      return bytes.remaining < 0 ? parentTag : undefined;
    }
    if (typeof value !== 'object' || value === null) return undefined;
    let tag = parentTag;
    let entered = false;
    try {
      if (value instanceof ArrayBuffer || ArrayBuffer.isView(value)) {
        bytes.remaining -= value.byteLength;
        return bytes.remaining < 0 ? tag : undefined;
      }
      if (
        value instanceof Map ||
        value instanceof Set ||
        value instanceof Date ||
        value instanceof RegExp ||
        (typeof Blob !== 'undefined' && value instanceof Blob)
      )
        return tag;
      const ownTag = Object.getOwnPropertyDescriptor(value, 'tag');
      const hasOwnTypeTag =
        ownTag !== undefined && Object.prototype.hasOwnProperty.call(ownTag, 'value') && typeof ownTag.value === 'string';
      if (hasOwnTypeTag) tag = ownTag.value;
      const nextTypeDepth = hasOwnTypeTag ? typeDepth + 1 : typeDepth;
      if (
        nextTypeDepth > MAX_SIGNATURE_DEPTH ||
        containerDepth > MAX_SNAPSHOT_CONTAINER_DEPTH ||
        active.has(value) ||
        budget.remaining-- <= 0
      )
        return tag;
      if (seen.has(value)) return undefined;
      active.add(value);
      entered = true;
      const keys = Reflect.ownKeys(value);
      if (Array.isArray(value)) {
        const length = Object.getOwnPropertyDescriptor(value, 'length');
        if (length === undefined || !isEncodedLength(length.value) || length.value > budget.remaining) return tag;
      }
      for (const key of keys) {
        if (budget.remaining-- <= 0) return tag;
        if (typeof key === 'string') {
          bytes.remaining -= key.length * 2;
          if (bytes.remaining < 0) return tag;
        }
        const descriptor = Object.getOwnPropertyDescriptor(value, key);
        if (descriptor === undefined || !Object.prototype.hasOwnProperty.call(descriptor, 'value')) return tag;
        const unreadable = inspect(descriptor.value, nextTypeDepth, containerDepth + 1, tag);
        if (unreadable !== undefined) return unreadable;
      }
      seen.add(value);
      return undefined;
    } catch {
      return tag;
    } finally {
      if (entered) active.delete(value);
    }
  };
  const unreadableTag = inspect(implemented, -1, 0, MALFORMED);
  if (unreadableTag !== undefined) return { unreadableTag };
  if (typeof globalThis.structuredClone !== 'function') return { unreadableTag: MALFORMED };
  try {
    return { value: globalThis.structuredClone(implemented) };
  } catch {
    // DataCloneError covers any Proxy in the graph, including a nested one whose traps could
    // otherwise alter a memoized signature between two uses.
    return { unreadableTag: MALFORMED };
  }
};

/** `value` as a readable record, or `undefined` if it is not one. */
const asRecord = (value: unknown): Record<string, unknown> | undefined =>
  typeof value === 'object' && value !== null && !Array.isArray(value) ? (value as Record<string, unknown>) : undefined;

// Generated signature objects use own data properties. Accessors can change values between a
// comparison and a memoized reuse, so they are not readable implementation metadata.
const ownData = (value: object, key: PropertyKey): boolean => {
  const descriptor = Object.getOwnPropertyDescriptor(value, key);
  return descriptor !== undefined && Object.prototype.hasOwnProperty.call(descriptor, 'value');
};

const REQUIRED_TYPE_FIELDS: { readonly [K in SignatureType['tag']]: readonly string[] } = {
  Boolean: [],
  Field: [],
  JubjubScalar: [],
  JubjubPoint: [],
  Secp256k1Base: [],
  Secp256k1Scalar: [],
  Secp256k1Point: [],
  Secp256r1Base: [],
  Secp256r1Scalar: [],
  Secp256r1Point: [],
  Curve25519Base: [],
  Curve25519Scalar: [],
  Curve25519Point: [],
  Uint: ['maxval'],
  Bytes: ['length'],
  Opaque: ['tsType'],
  Vector: ['length', 'type'],
  Tuple: ['types'],
  Enum: ['name', 'elements'],
  Struct: ['name', 'elements'],
  Alias: ['name', 'type'],
  Contract: ['name', 'circuits'],
};

/** A `Vector`'s element count as encoded: anything else makes the type unreadable, not unequal. */
const isEncodedLength = (value: unknown): boolean => typeof value === 'number' && Number.isSafeInteger(value) && value >= 0;

/**
 * The first tag in `t` this build can't read, or `undefined` if the whole type is readable.
 *
 * `t` is a {@link SignatureType} by declaration only — it came from a module we didn't compile — so
 * nothing here is trusted: the tag is read as `unknown`, and a known tag's payload is checked before
 * it is walked. A malformed payload reports the tag that carries it, which keeps a junk module
 * inside the `Unreadable` vocabulary instead of throwing a `TypeError` out of whichever field
 * happened to be read first.
 */
const firstUnreadableTag = (
  t: SignatureType,
  active = new Set<object>(),
  depth = 0,
  budget: ValidationBudget = { remaining: MAX_SIGNATURE_NODES },
): string | undefined => {
  const encoded = asRecord(t);
  if (encoded === undefined) {
    return MALFORMED;
  }
  if (!ownData(encoded, 'tag')) return MALFORMED;
  const tag = encoded.tag;
  if (typeof tag !== 'string') {
    return MALFORMED;
  }
  if (!Object.prototype.hasOwnProperty.call(KNOWN_SIGNATURE_TAGS, tag)) {
    return tag;
  }
  if (REQUIRED_TYPE_FIELDS[tag as SignatureType['tag']].some((key) => !ownData(encoded, key))) return tag;
  if (depth > MAX_SIGNATURE_DEPTH || active.has(encoded) || budget.remaining-- <= 0) {
    return tag;
  }
  active.add(encoded);
  try {
    switch (t.tag) {
      case 'Boolean':
      case 'Field':
      case 'JubjubScalar':
      case 'JubjubPoint':
      case 'Secp256k1Base':
      case 'Secp256k1Scalar':
      case 'Secp256k1Point':
      case 'Secp256r1Base':
      case 'Secp256r1Scalar':
      case 'Secp256r1Point':
      case 'Curve25519Base':
      case 'Curve25519Scalar':
      case 'Curve25519Point':
        return undefined;
      case 'Uint':
        return typeof t.maxval === 'string' && /^(0|[1-9][0-9]*)$/.test(t.maxval) ? undefined : t.tag;
      case 'Bytes':
        return isEncodedLength(t.length) ? undefined : t.tag;
      case 'Opaque':
        return typeof t.tsType === 'string' ? undefined : t.tag;
      // Its elements are names rather than types, so there is nothing to walk — but
      // `signatureTypesEqual` still reads them.
      case 'Enum': {
        const elements = t.elements;
        if (typeof t.name !== 'string' || !Array.isArray(elements) || elements.length > budget.remaining) return t.tag;
        budget.remaining -= elements.length;
        for (let i = 0; i < elements.length; i += 1) {
          if (!ownData(elements, i) || typeof elements[i] !== 'string') return t.tag;
        }
        return undefined;
      }
      case 'Alias':
        return typeof t.name === 'string' && asRecord(t.type) !== undefined
          ? firstUnreadableTag(t.type, active, depth + 1, budget)
          : t.tag;
      case 'Vector':
        return isEncodedLength(t.length) && asRecord(t.type) !== undefined
          ? firstUnreadableTag(t.type, active, depth + 1, budget)
          : t.tag;
      case 'Tuple':
        return Array.isArray(t.types) ? firstUnreadable(t.types, active, depth + 1, budget) : t.tag;
      case 'Struct': {
        const elements = t.elements;
        if (typeof t.name !== 'string' || !Array.isArray(elements)) return t.tag;
        for (let i = 0; i < elements.length; i += 1) {
          if (!ownData(elements, i)) return t.tag;
          const element = elements[i];
          const field = asRecord(element);
          if (
            field === undefined ||
            !ownData(field, 'name') ||
            !ownData(field, 'type') ||
            typeof field.name !== 'string' ||
            asRecord(field.type) === undefined
          )
            return t.tag;
          const unreadable = firstUnreadableTag(field.type as SignatureType, active, depth + 1, budget);
          if (unreadable !== undefined) return unreadable;
        }
        return undefined;
      }
      case 'Contract':
        return typeof t.name !== 'string' || asRecord(t.circuits) === undefined
          ? t.tag
          : firstUnreadableInDescriptor(t.circuits, active, depth + 1, budget);
      default: {
        // Unreachable: unknown tags returned above, known ones all have a case. A new variant lands
        // here until it gets one.
        const exhaustive: never = t;
        throw new CompactError(`unhandled signature type ${JSON.stringify(exhaustive)}`);
      }
    }
  } finally {
    active.delete(encoded);
  }
};

const firstUnreadable = (
  types: readonly SignatureType[],
  active: Set<object>,
  depth: number,
  budget: ValidationBudget,
): string | undefined => {
  for (let i = 0; i < types.length; i += 1) {
    if (!ownData(types, i)) return MALFORMED;
    const tag = firstUnreadableTag(types[i], active, depth, budget);
    if (tag !== undefined) {
      return tag;
    }
  }
  return undefined;
};

const firstUnreadableInDescriptor = (
  descriptor: InterfaceDescriptor,
  active: Set<object>,
  depth: number,
  budget: ValidationBudget,
): string | undefined => {
  if (depth > MAX_SIGNATURE_DEPTH || active.has(descriptor)) return MALFORMED;
  active.add(descriptor);
  try {
    // Only own circuit names are part of the encoded descriptor. `for...in` can walk an
    // arbitrarily large inherited prototype before it reaches any own name.
    for (const name of Object.keys(descriptor)) {
      if (budget.remaining-- <= 0) return MALFORMED;
      if (!ownData(descriptor, name)) return MALFORMED;
      const declaration = descriptor[name];
      if (
        asRecord(declaration) === undefined ||
        !ownData(declaration, 'argumentTypes') ||
        !ownData(declaration, 'resultType') ||
        !Array.isArray(declaration.argumentTypes)
      ) {
        return MALFORMED;
      }
      const tag = firstUnreadable(declaration.argumentTypes, active, depth + 1, budget);
      if (tag !== undefined) {
        return tag;
      }
      const resultTag = firstUnreadableTag(declaration.resultType, active, depth + 1, budget);
      if (resultTag !== undefined) {
        return resultTag;
      }
    }
    return undefined;
  } finally {
    active.delete(descriptor);
  }
};

/**
 * A sequence type's elements: how many, and which one is at an index.
 *
 * A view rather than an array because the length is compared first and is usually what differs. A
 * `Vector` carries its length as a number, so building one to measure it lets a
 * `{tag: 'Vector', length: 2 ** 30}` from an untrusted module allocate a billion slots against a
 * two-element tuple.
 */
type SequenceView = {
  readonly length: number;
  readonly at: (index: number) => SignatureType;
};

/**
 * A sequence type's elements, or `undefined` if this is not a sequence.
 *
 * The compiler identifies `Vector<n, T>` with an n-tuple of `T` (the `tvector`/`ttuple` cross-cases
 * in `sametype?`), so compare element lists rather than tags. The encoder canonicalizes both before
 * emission, so the cross-case should not arise.
 */
const sequenceElements = (t: SignatureType): SequenceView | undefined => {
  switch (t.tag) {
    case 'Vector':
      return { length: t.length, at: () => t.type };
    case 'Tuple':
      return { length: t.types.length, at: (index) => t.types[index] };
    default:
      return undefined;
  }
};

/**
 * Structural equality of two encoded contract-type circuit sets.
 *
 * Unlike `sametype?`, the contract type's name is not compared: the two sides come from different
 * compilation units, and the name is local to whichever one declared it.
 */
const contractCircuitsEqual = (a: InterfaceDescriptor, b: InterfaceDescriptor): boolean => {
  const aNames = Object.keys(a);
  if (aNames.length !== Object.keys(b).length) {
    return false;
  }
  return aNames.every((name) => {
    if (!Object.prototype.hasOwnProperty.call(b, name)) {
      return false;
    }
    const x = a[name];
    const y = b[name];
    return (
      x.argumentTypes.length === y.argumentTypes.length &&
      x.argumentTypes.every((t, i) => signatureTypesEqual(t, y.argumentTypes[i])) &&
      signatureTypesEqual(x.resultType, y.resultType)
    );
  });
};

/**
 * Whether two encoded Compact types are the same type.
 *
 * Nominal for structs, enums and `new type` aliases, matching the compiler; structural for contract
 * types, per {@link contractCircuitsEqual}.
 *
 * The name is all the nominal cases have. Nothing in the encoding records which module declared it,
 * so two compilation units that each write `new type Meters = Uint<64>` are one type here. The
 * underlying type is still compared, so an alias collision cannot smuggle a different
 * representation past — but a struct agreeing in name and fields is accepted whatever it meant.
 */
export function signatureTypesEqual(a: SignatureType, b: SignatureType): boolean {
  if (a.tag === 'Vector' && b.tag === 'Vector') {
    return a.length === b.length && (a.length === 0 || signatureTypesEqual(a.type, b.type));
  }
  const aSeq = sequenceElements(a);
  if (aSeq !== undefined) {
    const bSeq = sequenceElements(b);
    if (bSeq === undefined || aSeq.length !== bSeq.length) {
      return false;
    }
    for (let index = 0; index < aSeq.length; index += 1) {
      if (!signatureTypesEqual(aSeq.at(index), bSeq.at(index))) {
        return false;
      }
    }
    return true;
  }
  switch (a.tag) {
    case 'Boolean':
    case 'Field':
    case 'JubjubScalar':
    case 'JubjubPoint':
    case 'Secp256k1Base':
    case 'Secp256k1Scalar':
    case 'Secp256k1Point':
    case 'Secp256r1Base':
    case 'Secp256r1Scalar':
    case 'Secp256r1Point':
    case 'Curve25519Base':
    case 'Curve25519Scalar':
    case 'Curve25519Point':
      return b.tag === a.tag;
    case 'Uint':
      return b.tag === 'Uint' && a.maxval === b.maxval;
    case 'Bytes':
      return b.tag === 'Bytes' && a.length === b.length;
    case 'Opaque':
      return b.tag === 'Opaque' && a.tsType === b.tsType;
    case 'Enum':
      return (
        b.tag === 'Enum' &&
        a.name === b.name &&
        a.elements.length === b.elements.length &&
        a.elements.every((e, i) => e === b.elements[i])
      );
    case 'Struct':
      return (
        b.tag === 'Struct' &&
        a.name === b.name &&
        a.elements.length === b.elements.length &&
        a.elements.every((e, i) => e.name === b.elements[i].name && signatureTypesEqual(e.type, b.elements[i].type))
      );
    case 'Alias':
      return b.tag === 'Alias' && a.name === b.name && signatureTypesEqual(a.type, b.type);
    case 'Contract':
      return b.tag === 'Contract' && contractCircuitsEqual(a.circuits, b.circuits);
    case 'Vector':
    case 'Tuple':
      // Unreachable: sequences are compared above, where a Vector and a Tuple
      // over the same element types are identified as the compiler does.
      return false;
    default: {
      // Only reachable with a tag outside the union, which `checkConformance` screens for first.
      // Throw rather than return `false`, which would read as "these types differ".
      const exhaustive: never = a;
      throw new CompactError(`unreadable signature type ${JSON.stringify(exhaustive)}`);
    }
  }
}

type TypeCheck = { readonly unreadableTag?: string; readonly equal: boolean };
type TypeCheckState = {
  readonly active: Set<object>;
  readonly memo: Map<object, Map<object, Map<number, TypeCheck>>>;
  readonly budget: ValidationBudget;
};

// Compare within the shape declared by the caller. A matching expanded Vector is visited element
// by element; an unrelated shape falls back to the fixed local validation allowance. Within a
// matching shape, keep scanning after a difference so a later unreadable tag takes precedence.
// The total allowance is derived from the trusted declaration's logical shape, while memoization
// prevents a shared implementation DAG from multiplying the amount of work.
const inspectTypePair = (declared: SignatureType, implemented: SignatureType, state: TypeCheckState, depth = 0): TypeCheck => {
  const actual = asRecord(implemented);
  if (actual === undefined || !ownData(actual, 'tag') || typeof actual.tag !== 'string')
    return { unreadableTag: MALFORMED, equal: false };
  const tag = actual.tag;
  if (!Object.prototype.hasOwnProperty.call(KNOWN_SIGNATURE_TAGS, tag)) return { unreadableTag: tag, equal: false };
  if (REQUIRED_TYPE_FIELDS[tag as SignatureType['tag']].some((key) => !ownData(actual, key)))
    return { unreadableTag: tag, equal: false };
  if (depth > MAX_SIGNATURE_DEPTH || state.active.has(actual)) return { unreadableTag: tag, equal: false };
  const cached = state.memo.get(declared)?.get(actual)?.get(depth);
  if (cached !== undefined) return cached;

  const fallback = (): TypeCheck => {
    // The fallback validates this node again, so replace the entry charge rather than counting it twice.
    state.budget.remaining += 1;
    const local = validationBudget(declared);
    local.remaining = Math.min(local.remaining, state.budget.remaining);
    const before = local.remaining;
    const unreadableTag = firstUnreadableTag(implemented, new Set<object>(), depth, local);
    state.budget.remaining -= before - local.remaining;
    return unreadableTag === undefined ? { equal: signatureTypesEqual(declared, implemented) } : { unreadableTag, equal: false };
  };
  const inspectChildren = (
    length: number,
    expectedAt: (index: number) => SignatureType,
    receivedAt: (index: number) => SignatureType,
    childDepth: number,
    receivedArray?: readonly unknown[],
  ): TypeCheck => {
    let equal = true;
    for (let i = 0; i < length; i += 1) {
      if (receivedArray !== undefined && !ownData(receivedArray, i)) return { unreadableTag: tag, equal: false };
      const child = inspectTypePair(expectedAt(i), receivedAt(i), state, childDepth);
      if (child.unreadableTag !== undefined) return child;
      if (!child.equal) equal = false;
    }
    return { equal };
  };
  state.active.add(actual);
  const compute = (): TypeCheck => {
    if (state.budget.remaining-- <= 0) return { unreadableTag: tag, equal: false };
    if (
      declared.tag === 'Vector' &&
      implemented.tag === 'Vector' &&
      isEncodedLength(implemented.length) &&
      asRecord(implemented.type) !== undefined &&
      declared.length === implemented.length
    ) {
      const child = inspectTypePair(declared.type, implemented.type, state, depth + 1);
      return child.unreadableTag === undefined && declared.length === 0 ? { equal: true } : child;
    }
    if (
      declared.tag === 'Vector' &&
      implemented.tag === 'Tuple' &&
      Array.isArray(implemented.types) &&
      implemented.types.length === declared.length
    ) {
      return inspectChildren(
        implemented.types.length,
        () => declared.type,
        (i) => implemented.types[i],
        depth + 1,
        implemented.types,
      );
    }
    if (
      declared.tag === 'Tuple' &&
      implemented.tag === 'Tuple' &&
      Array.isArray(implemented.types) &&
      implemented.types.length === declared.types.length
    ) {
      return inspectChildren(
        declared.types.length,
        (i) => declared.types[i],
        (i) => implemented.types[i],
        depth + 1,
        implemented.types,
      );
    }
    if (
      declared.tag === 'Tuple' &&
      implemented.tag === 'Vector' &&
      isEncodedLength(implemented.length) &&
      asRecord(implemented.type) !== undefined &&
      declared.types.length === implemented.length &&
      declared.types.length > 0
    ) {
      return inspectChildren(
        declared.types.length,
        (i) => declared.types[i],
        () => implemented.type,
        depth + 1,
      );
    }
    if (
      declared.tag === 'Alias' &&
      implemented.tag === 'Alias' &&
      typeof implemented.name === 'string' &&
      asRecord(implemented.type) !== undefined
    ) {
      const child = inspectTypePair(declared.type, implemented.type, state, depth + 1);
      return child.unreadableTag === undefined ? { equal: declared.name === implemented.name && child.equal } : child;
    }
    if (
      declared.tag === 'Struct' &&
      implemented.tag === 'Struct' &&
      typeof implemented.name === 'string' &&
      Array.isArray(implemented.elements) &&
      declared.elements.length === implemented.elements.length
    ) {
      let equal = declared.name === implemented.name;
      for (let i = 0; i < declared.elements.length; i += 1) {
        if (!ownData(implemented.elements, i)) return { unreadableTag: 'Struct', equal: false };
        const field = asRecord(implemented.elements[i]);
        if (
          field === undefined ||
          !ownData(field, 'name') ||
          !ownData(field, 'type') ||
          typeof field.name !== 'string' ||
          asRecord(field.type) === undefined
        )
          return { unreadableTag: 'Struct', equal: false };
        if (field.name !== declared.elements[i].name) equal = false;
        const child = inspectTypePair(declared.elements[i].type, field.type as SignatureType, state, depth + 1);
        if (child.unreadableTag !== undefined) return child;
        if (!child.equal) equal = false;
      }
      return { equal };
    }
    if (
      declared.tag === 'Contract' &&
      implemented.tag === 'Contract' &&
      typeof implemented.name === 'string' &&
      asRecord(implemented.circuits) !== undefined
    ) {
      const expectedNames = Object.keys(declared.circuits);
      const actualNames = Object.keys(implemented.circuits);
      if (
        actualNames.length !== expectedNames.length ||
        actualNames.some((name) => !Object.prototype.hasOwnProperty.call(declared.circuits, name))
      )
        return fallback();
      let equal = true;
      for (const name of expectedNames) {
        const expected = declared.circuits[name];
        if (!ownData(implemented.circuits, name)) return { unreadableTag: 'Contract', equal: false };
        const received = implemented.circuits[name];
        const record = asRecord(received);
        if (
          record === undefined ||
          !ownData(record, 'argumentTypes') ||
          !ownData(record, 'resultType') ||
          !Array.isArray(received.argumentTypes)
        )
          return { unreadableTag: MALFORMED, equal: false };
        if (received.argumentTypes.length !== expected.argumentTypes.length) return fallback();
        const argumentsResult = inspectChildren(
          expected.argumentTypes.length,
          (i) => expected.argumentTypes[i],
          (i) => received.argumentTypes[i],
          depth + 2,
          received.argumentTypes,
        );
        if (argumentsResult.unreadableTag !== undefined) return argumentsResult;
        if (!argumentsResult.equal) equal = false;
        const result = inspectTypePair(expected.resultType, received.resultType, state, depth + 2);
        if (result.unreadableTag !== undefined) return result;
        if (!result.equal) equal = false;
      }
      return { equal };
    }
    return fallback();
  };
  let result: TypeCheck;
  try {
    result = compute();
  } finally {
    state.active.delete(actual);
  }
  // A fully inspected readable subtree is cycle-free: any path back to this node or an active
  // ancestor would have returned Unreadable. Its equality result is independent of the ancestor
  // path, including when a diamond reaches the same node. A successful fallback is reusable too;
  // an unreadable or budget-exhausted result is not.
  if (result.unreadableTag === undefined) {
    let byActual = state.memo.get(declared);
    if (byActual === undefined) {
      byActual = new Map();
      state.memo.set(declared, byActual);
    }
    let byDepth = byActual.get(actual);
    if (byDepth === undefined) {
      byDepth = new Map();
      byActual.set(actual, byDepth);
    }
    byDepth.set(depth, result);
  }
  return result;
};

const inspectAgainstDeclaration = (declared: SignatureType, implemented: SignatureType): TypeCheck =>
  inspectTypePair(declared, implemented, {
    active: new Set<object>(),
    memo: new Map<object, Map<object, Map<number, TypeCheck>>>(),
    budget: validationBudget(declared, true),
  });

/**
 * Check a resolved module against the contract type its caller declared, returning the first thing
 * that went wrong, or `Conformant`.
 *
 * A module may implement more circuits than the declaration names, but not fewer and not
 * differently. A declared circuit must have a proof, so its implementation must be provable; a pure
 * implementation is not.
 *
 * Only the implementation is screened for readability; the declaration comes from the caller's own
 * module, which is version-locked to this runtime.
 *
 * @param declaration The caller's `declaredInterfaces[T]` for the contract type being called through.
 * @param implementation The resolved module's `circuitSignatures`.
 */
export function checkConformance(declaration: InterfaceDescriptor, implementation: CircuitSignatures): ConformanceResult {
  const declaredCircuitIds = Object.keys(declaration);
  if (declaredCircuitIds.length === 0) return { outcome: 'Conformant' };
  // Detach every circuit before comparing the first one: a later resultType getter must not be
  // able to mutate an earlier argument after that argument has already passed conformance.
  const descriptorDeclaration: SignatureType = { tag: 'Contract', name: '(implementation)', circuits: declaration };
  const snapshot = snapshotSignature(descriptorDeclaration, implementation as unknown as SignatureType);
  if (snapshot.unreadableTag !== undefined || snapshot.value === undefined) {
    // The whole-descriptor snapshot is the authority. Locate the unreadable argument/result for
    // diagnostics only; this path never compares or accepts the live implementation.
    try {
      for (const circuitId of declaredCircuitIds) {
        const circuitProperty = Object.getOwnPropertyDescriptor(implementation, circuitId);
        if (circuitProperty === undefined) return { outcome: 'Violation', circuitId, check: 'Existence' };
        if (!Object.prototype.hasOwnProperty.call(circuitProperty, 'value'))
          return { outcome: 'Unreadable', circuitId, unreadableTag: MALFORMED };
        const circuit = asRecord(circuitProperty.value);
        if (circuit === undefined || !ownData(circuit, 'argumentTypes') || !ownData(circuit, 'resultType'))
          return { outcome: 'Unreadable', circuitId, unreadableTag: MALFORMED };
        const argumentsValue = circuit.argumentTypes;
        if (!Array.isArray(argumentsValue)) return { outcome: 'Unreadable', circuitId, unreadableTag: MALFORMED };
        if (!ownData(circuit, 'pure') || !ownData(circuit, 'provable'))
          return { outcome: 'Unreadable', circuitId, unreadableTag: MALFORMED };
        if (circuit.provable !== true || circuit.pure !== false) return { outcome: 'Violation', circuitId, check: 'Provability' };
        const expected = declaration[circuitId];
        if (expected.argumentTypes.length !== argumentsValue.length) return { outcome: 'Violation', circuitId, check: 'Arity' };
        for (let i = 0; i < Math.min(expected.argumentTypes.length, argumentsValue.length); i += 1) {
          if (!ownData(argumentsValue, i))
            return { outcome: 'Unreadable', circuitId, unreadableTag: MALFORMED, argumentIndex: i };
          const located = snapshotSignature(expected.argumentTypes[i], argumentsValue[i] as SignatureType);
          if (located.unreadableTag !== undefined || located.value === undefined)
            return { outcome: 'Unreadable', circuitId, unreadableTag: located.unreadableTag ?? MALFORMED, argumentIndex: i };
          const checked = inspectAgainstDeclaration(expected.argumentTypes[i], located.value);
          if (checked.unreadableTag !== undefined)
            return { outcome: 'Unreadable', circuitId, unreadableTag: checked.unreadableTag, argumentIndex: i };
          if (!checked.equal) return { outcome: 'Violation', circuitId, check: 'ArgumentType', argumentIndex: i };
        }
        const result = snapshotSignature(expected.resultType, circuit.resultType as SignatureType);
        if (result.unreadableTag !== undefined || result.value === undefined)
          return { outcome: 'Unreadable', circuitId, unreadableTag: result.unreadableTag ?? MALFORMED };
        const checkedResult = inspectAgainstDeclaration(expected.resultType, result.value);
        if (checkedResult.unreadableTag !== undefined)
          return { outcome: 'Unreadable', circuitId, unreadableTag: checkedResult.unreadableTag };
        if (!checkedResult.equal) return { outcome: 'Violation', circuitId, check: 'ResultType' };
      }
    } catch {
      // Hostile Proxy traps can also throw while locating a diagnostic.
    }
    return { outcome: 'Unreadable', circuitId: declaredCircuitIds[0], unreadableTag: snapshot.unreadableTag ?? MALFORMED };
  }
  const detachedImplementation = snapshot.value as unknown as CircuitSignatures;
  for (const circuitId of declaredCircuitIds) {
    const declared = declaration[circuitId];
    // Own-property, not `in` or a bare index: `constructor` and `toString` are legal circuit names,
    // and `Object.prototype`'s members would otherwise read as implementations.
    if (!Object.prototype.hasOwnProperty.call(detachedImplementation, circuitId)) {
      return { outcome: 'Violation', circuitId, check: 'Existence' };
    }
    const implemented = detachedImplementation[circuitId];
    // Also a module we didn't compile. Its argument list must be readable before arity is checked.
    if (!Array.isArray(asRecord(implemented)?.argumentTypes)) {
      return { outcome: 'Unreadable', circuitId, unreadableTag: MALFORMED };
    }
    if (implemented.provable !== true || implemented.pure !== false) {
      return { outcome: 'Violation', circuitId, check: 'Provability' };
    }
    if (declared.argumentTypes.length !== implemented.argumentTypes.length) {
      return { outcome: 'Violation', circuitId, check: 'Arity' };
    }
    for (let i = 0; i < declared.argumentTypes.length; i += 1) {
      const snapshot = snapshotSignature(declared.argumentTypes[i], implemented.argumentTypes[i]);
      if (snapshot.unreadableTag !== undefined || snapshot.value === undefined)
        return { outcome: 'Unreadable', circuitId, unreadableTag: snapshot.unreadableTag ?? MALFORMED, argumentIndex: i };
      const checked = inspectAgainstDeclaration(declared.argumentTypes[i], snapshot.value);
      if (checked.unreadableTag !== undefined) {
        return { outcome: 'Unreadable', circuitId, unreadableTag: checked.unreadableTag, argumentIndex: i };
      }
      if (!checked.equal) return { outcome: 'Violation', circuitId, check: 'ArgumentType', argumentIndex: i };
    }
    const snapshotResult = snapshotSignature(declared.resultType, implemented.resultType);
    if (snapshotResult.unreadableTag !== undefined || snapshotResult.value === undefined)
      return { outcome: 'Unreadable', circuitId, unreadableTag: snapshotResult.unreadableTag ?? MALFORMED };
    const checkedResult = inspectAgainstDeclaration(declared.resultType, snapshotResult.value);
    if (checkedResult.unreadableTag !== undefined) {
      return { outcome: 'Unreadable', circuitId, unreadableTag: checkedResult.unreadableTag };
    }
    if (!checkedResult.equal) return { outcome: 'Violation', circuitId, check: 'ResultType' };
  }
  return { outcome: 'Conformant' };
}
