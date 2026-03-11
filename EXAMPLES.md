## Example documents (accompanying the spec)

### 1) Profile Document (standalone JSON‑LD)
Example URL: `https://joeblogs/profile.jsonld`

```textmate
{
  "@context": "https://schema.org",
  "@type": "Person",
  "@id": "https://joeblogs/profile.jsonld#me",
  "name": "Joe Blogs",
  "url": "https://joeblogs/",
  "email": "mailto:ada@[example].com",
  "telephone": "+[country-code]-[number]",
  "address": {
    "@type": "PostalAddress",
    "streetAddress": "[street]",
    "addressLocality": "[city]",
    "addressRegion": "[region/state]",
    "postalCode": "[postal-code]",
    "addressCountry": "[country-code]"
  },
  "identifier": [
    {
      "@type": "PropertyValue",
      "propertyID": "blsttc:public-key",
      "value": "https://joeblogs/keys/blsttc/2026-02-28/public-key.json"
    },
    {
      "@type": "PropertyValue",
      "propertyID": "blsttc:public-key-fingerprint",
      "value": "sha256:[hex_sha256_of_canonical_key_bytes]"
    }
  ]
}
```

---

### 2) Optional: Profile Document variant without direct email/phone
If you prefer less scrapeable contact info, keep the profile but omit `email` and `telephone`:

```textmate
{
  "@context": "https://schema.org",
  "@type": "Person",
  "@id": "https://joeblogs/profile.jsonld#me",
  "name": "Joe Blogs",
  "url": "https://joeblogs/",
  "identifier": [
    {
      "@type": "PropertyValue",
      "propertyID": "blsttc:public-key",
      "value": "https://joeblogs/keys/blsttc/2026-02-28/public-key.json"
    },
    {
      "@type": "PropertyValue",
      "propertyID": "blsttc:public-key-fingerprint",
      "value": "sha256:[hex_sha256_of_canonical_key_bytes]"
    }
  ]
}
```
