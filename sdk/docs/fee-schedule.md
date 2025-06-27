# Endpoints that charge fees for their use

All endpoints that change the state charge a validation fee.

`vsl_submitClaim` will transfer to __each__ of the verifiers the verification fee.

| Endpoint | Validation fee | Verification fee |
| -------- | -------------- | ---------------- |
| vsl_settleClaim | yes | |
| vsl_submitClaim | yes | yes |
| vsl_pay | yes | |
| vsl_createAsset | yes | |
| vsl_transferAsset | yes | |
| vsl_setAccountState | yes | |

