  (test
    '(
      "import CompactStandardLibrary;"
      "export ledger scalar: Curve25519Scalar;"
      "export circuit test(bytes: Bytes<64>): Curve25519Scalar {"
      "  const s = bytes as Curve25519Scalar;"
      "  scalar = disclose(s);"
      "  return s;"
      "}"
      )
    (pass-returns reduce-to-zkir
      (program
        (circuit (test) ((%bytes.0 "Scalar<BLS12-381>")
                         (%bytes.1 "Scalar<BLS12-381>")
                         (%bytes.2 "Scalar<BLS12-381>"))
          ("Scalar<Curve25519>")
          (constrain_bits %bytes.0 16)
          (constrain_bits %bytes.1 248)
          (constrain_bits %bytes.2 248)
          (bytes_from_natives %tmp.3 64 %bytes.2 %bytes.1 %bytes.0)
          (from_bytes "Scalar<Curve25519>" %s.4 %tmp.3)
          (encode (%fld.5 %fld.6) %s.4)
          (impact 1 16 1 1 1 0)
          (impact 1 17 1 2 26 7 %fld.5 %fld.6)
          (impact 1 145)
          (output %s.4))))
    (stage-javascript
      '("test('Bytes<64> cast to Curve25519Scalar', async () => {"
        "  const [contract, context] = await startContract(contractCode, {}, 0);"
        "  // Random value in range."
        "  var scalar = 0x0e7545706a590d3b6d348e5e55058f0808d30736058953ee40bc238694733d9an;"
        "  var bytes = new Uint8Array(64);"
        "  bytes.set(["
        "    0x9a, 0x3d, 0x73, 0x94, 0x86, 0x23, 0xbc, 0x40,"
        "    0xee, 0x53, 0x89, 0x05, 0x36, 0x07, 0xd3, 0x08,"
        "    0x08, 0x8f, 0x05, 0x55, 0x5e, 0x8e, 0x34, 0x6d,"
        "    0x3b, 0x0d, 0x59, 0x6a, 0x70, 0x45, 0x75, 0x0e,"
        "  ], 0);"
        "  var r = await contract.circuits.test(context, bytes);"
        "  expect(r.result).toEqual(scalar);"
        "  expect(contractCode.ledger(r.context.callContext.currentQueryContext.state).scalar).toEqual(scalar);"
        "  // Zero."
        "  bytes = new Uint8Array(64);"
        "  r = await contract.circuits.test(context, bytes);"
        "  expect(r.result).toEqual(0n);"
        "  expect(contractCode.ledger(r.context.callContext.currentQueryContext.state).scalar).toEqual(0n);"
        "  // Maximum value."
        "  bytes = new Uint8Array(64);"
        "  bytes.set(["
        "    0xec, 0xd3, 0xf5, 0x5c, 0x1a, 0x63, 0x12, 0x58,"
        "    0xd6, 0x9c, 0xf7, 0xa2, 0xde, 0xf9, 0xde, 0x14,"
        "    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,"
        "    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10,"
        "  ], 0);"
        "  r = await contract.circuits.test(context, bytes);"
        "  expect(r.result).toEqual(runtime.MAX_CURVE25519_SCALAR);"
        "  expect(contractCode.ledger(r.context.callContext.currentQueryContext.state).scalar).toEqual("
        "      runtime.MAX_CURVE25519_SCALAR);"
        "  // Maximum value plus one."
        "  bytes[0] += 1;"
        "  r = await contract.circuits.test(context, bytes);"
        "  expect(r.result).toEqual(0n);"
        "  expect(contractCode.ledger(r.context.callContext.currentQueryContext.state).scalar).toEqual(0n);"
        "  // This is scalar * 10^42 in the field."
        "  bytes = new Uint8Array(["
        "    0x9a, 0x3d, 0x73, 0x94, 0x86, 0x37, 0x60, 0xbc,"
        "    0x8d, 0x22, 0x78, 0x88, 0x98, 0xaf, 0x00, 0x71,"
        "    0x07, 0xd2, 0x99, 0xf6, 0xc2, 0xd6, 0x37, 0x66,"
        "    0x6e, 0xef, 0xef, 0xf5, 0x8d, 0xb0, 0xd0, 0xa4,"
        "    0xef, 0x00, 0x00, 0x00, 0x40, 0x9e, 0x3d, 0x4a,"
        "    0xf1, 0xad, 0x05, 0x03, 0x05, 0x27, 0xc6, 0xab,"
        "    0xb7, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,"
        "    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,"
        "  ]);"
        "  r = await contract.circuits.test(context, bytes);"
        "  expect(r.result).toEqual(scalar);"
        "  expect(contractCode.ledger(r.context.callContext.currentQueryContext.state).scalar).toEqual(scalar);"
        "});"
        ))
  )
