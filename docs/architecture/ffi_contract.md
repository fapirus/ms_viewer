# FFI Contract

## Goal
Rust 엔진과 Flutter 패키지 사이의 데이터 교환 형식을 정의한다.

## Design rules
- FFI 경계에서는 포맷별 내부 구조를 직접 노출하지 않는다.
- Flutter는 플랫폼 채널이나 직접 FFI 호출 결과를 동일한 계약 모델로 다룬다.
- 에러는 문자열이 아니라 코드 기반으로 전달한다.
- 대용량 문서를 위해 문서 전체를 한 번에 넘기지 않고 document/session 기반으로 접근한다.

## Initial contract

### Open document request
```json
{
  "source": {
    "kind": "path",
    "value": "/absolute/path/sample.docx"
  },
  "options": {
    "password": null,
    "preferLazyLoading": true
  }
}
```

### Open document success response
```json
{
  "documentId": "doc_001",
  "kind": "docx",
  "title": "sample.docx",
  "pageCount": 12,
  "capabilities": {
    "search": true,
    "textSelection": true,
    "passwordProtected": false
  }
}
```

### Open document error response
```json
{
  "code": "password_required",
  "message": "Password is required to open this document."
}
```

## Error codes
- `unsupported_format`
- `password_required`
- `invalid_password`
- `unsupported_encryption`
- `io_error`
- `invalid_document`
- `not_implemented`

## Next APIs
- `openDocument`
- `closeDocument`
- `getPage`
- `searchDocument`
- `resolveSelection`
- `submitPassword`
