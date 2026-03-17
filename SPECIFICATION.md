## Specification: Web Profile JSON‑LD with Immutable `blsttc` Public Key Reference (v1)

### 1. Scope
This specification defines:
- A **standalone JSON‑LD profile document** that represents an individual’s contact information.
- A separate, **immutable public key document** for a single `blsttc` public key.
- A link mechanism from the profile to the key document suitable for web publishing.

### 2. Terminology
- **Profile Document**: the JSON‑LD document describing a person.
- **Key Document**: a JSON (or JSON‑LD) document containing a single `blsttc` public key and metadata.
- **Canonical Key Bytes**: the exact byte sequence representing the public key used for fingerprinting (defined by the publisher).

### 3. Publishing and Transport Requirements
3.1 **Profile Document**
- MUST be served as a **standalone** resource (e.g., `/profile.jsonld`).
- SHOULD use HTTP `Content-Type: application/ld+json; charset=utf-8`.
- MUST use the Schema.org context:
    - `@context` MUST be `"https://schema.org"`.

3.2 **Key Document**
- MUST be served at a **stable, immutable URL**.
- MUST NOT change once published at that URL.
- SHOULD be cacheable long-term (e.g., `Cache-Control: public, max-age=31536000, immutable`).

### 4. Profile Document Format (JSON‑LD)
4.1 Required fields
The Profile Document MUST contain:
- `@context`: `"https://schema.org"`
- `@type`: `"Person"`
- `@id`: a stable URI identifying the person (recommended: a fragment under the profile URL, e.g. `.../profile.jsonld#me`)
- `name`: a human-readable name
- `url`: the person’s primary homepage/profile URL

4.2 Contact fields (optional)
The Profile Document MAY contain:
- `email`: a `mailto:` URI
- `telephone`: a telephone number as a string
- `address`: a `PostalAddress` object with any of:
    - `streetAddress`, `addressLocality`, `addressRegion`, `postalCode`, `addressCountry`

4.3 `blsttc` public key references
The Profile Document MUST reference the `blsttc` public key using `identifier` entries of type `PropertyValue`.

- `identifier` MUST be an array.
- For a single `blsttc` key, the Profile Document MUST include:
    1) A key URL reference:
        - `@type`: `"PropertyValue"`
        - `propertyID`: `"blsttc:public-key"`
        - `value`: the immutable Key Document URL (HTTPS recommended)
    2) A fingerprint reference (recommended, but if you want it mandatory: make it MUST in your implementation):
        - `@type`: `"PropertyValue"`
        - `propertyID`: `"blsttc:public-key-fingerprint"`
        - `value`: a string formatted as `sha256:<hex>` representing the SHA‑256 of the Canonical Key Bytes

### 5. Key Document Format (single-key, immutable)
5.1 Required fields
A Key Document MUST include:
- `type`: MUST be `"blsttc-public-key"`
- `format`: MUST be `"blsttc"`
- `curve`: the curve identifier (e.g., `"BLS12-381"`)
- `encoding`: the encoding used for `publicKey` (this spec defines `"hex"`)
- `publicKey`: the public key encoded according to `encoding`
- `created`: an ISO 8601 timestamp indicating publication/creation time
- `id`: the canonical URL of this Key Document (SHOULD equal the URL used to fetch it)

5.2 Fingerprints
The Key Document MUST include a `fingerprints` object containing:
- `sha256`: hex-encoded SHA‑256 digest of the **Canonical Key Bytes**

### 6. Canonicalization and Fingerprinting Rules
6.1 Canonical Key Bytes definition
The publisher MUST define what byte representation is used as **Canonical Key Bytes**. At minimum, the definition MUST specify:
- Whether the key bytes are **compressed or uncompressed**
- The exact serialization method/library convention used
- That the fingerprint input is the **raw key bytes**, not the JSON text

6.2 Fingerprint algorithm
- SHA‑256 MUST be computed over the Canonical Key Bytes.
- The resulting digest MUST be lowercase hex (recommended) or explicitly specify casing in your policy and stick to it.

### 7. Immutability and Rotation
- Each published Key Document URL MUST be immutable.
- Key rotation MUST be performed by publishing a **new** Key Document at a **new** URL and updating the Profile Document to reference the new URL (or publishing a separate mutable “latest” pointer document if desired).

### 8. Security and Privacy Considerations
- Publishing `email` and `telephone` increases scrapeability; publishers SHOULD omit or obfuscate these if spam/harassment is a concern.
- Consumers SHOULD validate that:
    - The fetched Key Document URL matches the profile’s referenced URL
    - The computed `sha256` of Canonical Key Bytes matches both the Profile fingerprint (if present) and the Key Document fingerprint

### 9. Example Structures (non-normative)
- Profile Document: Schema.org `Person` + `identifier[]` `PropertyValue` references for key URL and fingerprint.
- Key Document: JSON object containing `publicKey` (hex), `curve`, `created`, `id`, and `fingerprints.sha256`.
