// Generated by the following commands:
// > openssl ecparam -name prime256v1 -genkey -noout -out key.pem
// > openssl req -new -sha256 -x509 -days 3652 -key key.pem -subj "/CN=Soft U2F" -out certificate.pem
// > openssl x509 -text -noout -in certificate.pem

// Publishing this private key means attestation for the corresponding
// certificate cannot be trusted, i.e. the relying party cannot
// prove a registration came from this particular U2F "device" implementation.
// The benefit of sharing this key between "devices" verus generating
// a per-"device" secret is thwarting user-tracking.
// Further, at the current time there is no chain of trust for attestation certficates,
// nor do any sites seem to have lists of specific allowed attestation certificates,
// so there is little lost by publicizing this key.
pub const SELF_SIGNED_ATTESTATION_KEY_PEM: &str = "-----BEGIN EC PRIVATE KEY-----
MHcCAQEEINEOLIK0c4FmXL3ImqB65YV63JyaR3NGWA7ShLmL6GfboAoGCCqGSM49
AwEHoUQDQgAEOYIz5kgxQSWqea8AzHcqjuJQnFqkF8V7RwTfkvs7esrQt2pqrqHL
VxrjjimYpE5E/4F/CxV1apjGSHLwuBM4hg==
-----END EC PRIVATE KEY-----";

// Certificate:
//     Data:
//         Version: 3 (0x2)
//         Serial Number:
//             8a:8a:f7:75:c2:3a:bf:c6
//     Signature Algorithm: ecdsa-with-SHA256
//         Issuer: CN = Soft U2F
//         Validity
//             Not Before: Oct 20 21:51:33 2017 GMT
//             Not After : Oct 20 21:51:33 2027 GMT
//         Subject: CN = Soft U2F
//         Subject Public Key Info:
//             Public Key Algorithm: id-ecPublicKey
//                 Public-Key: (256 bit)
//                 pub:
//                     04:39:82:33:e6:48:31:41:25:aa:79:af:00:cc:77:
//                     2a:8e:e2:50:9c:5a:a4:17:c5:7b:47:04:df:92:fb:
//                     3b:7a:ca:d0:b7:6a:6a:ae:a1:cb:57:1a:e3:8e:29:
//                     98:a4:4e:44:ff:81:7f:0b:15:75:6a:98:c6:48:72:
//                     f0:b8:13:38:86
//                 ASN1 OID: prime256v1
//                 NIST CURVE: P-256
//         X509v3 extensions:
//             X509v3 Subject Key Identifier:
//                 F1:B2:70:DA:2D:41:8E:A2:36:BA:FA:90:5A:A3:5E:9A:9C:53:C4:3E
//             X509v3 Authority Key Identifier:
//                 keyid:F1:B2:70:DA:2D:41:8E:A2:36:BA:FA:90:5A:A3:5E:9A:9C:53:C4:3E

//             X509v3 Basic Constraints: critical
//                 CA:TRUE
//     Signature Algorithm: ecdsa-with-SHA256
//          30:46:02:21:00:a4:57:5c:9e:f7:f3:89:a0:2e:9e:57:64:02:
//          f1:c3:c0:d5:62:44:7e:3a:d5:f6:6f:ff:ab:45:95:b6:0f:18:
//          4c:02:21:00:92:d4:f3:3c:7c:c3:8a:4e:07:6b:de:1a:6a:79:
//          0a:bf:ca:c4:0a:f0:d2:59:b4:f6:c5:00:31:24:f1:e7:df:1d
pub const SELF_SIGNED_ATTESTATION_CERTIFICATE_PEM: &str = "-----BEGIN CERTIFICATE-----
MIIBcTCCARagAwIBAgIJAIqK93XCOr/GMAoGCCqGSM49BAMCMBMxETAPBgNVBAMM
CFNvZnQgVTJGMB4XDTE3MTAyMDIxNTEzM1oXDTI3MTAyMDIxNTEzM1owEzERMA8G
A1UEAwwIU29mdCBVMkYwWTATBgcqhkjOPQIBBggqhkjOPQMBBwNCAAQ5gjPmSDFB
Jap5rwDMdyqO4lCcWqQXxXtHBN+S+zt6ytC3amquoctXGuOOKZikTkT/gX8LFXVq
mMZIcvC4EziGo1MwUTAdBgNVHQ4EFgQU8bJw2i1BjqI2uvqQWqNempxTxD4wHwYD
VR0jBBgwFoAU8bJw2i1BjqI2uvqQWqNempxTxD4wDwYDVR0TAQH/BAUwAwEB/zAK
BggqhkjOPQQDAgNJADBGAiEApFdcnvfziaAunldkAvHDwNViRH461fZv/6tFlbYP
GEwCIQCS1PM8fMOKTgdr3hpqeQq/ysQK8NJZtPbFADEk8effHQ==
-----END CERTIFICATE-----";
