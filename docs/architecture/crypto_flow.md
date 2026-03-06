# Crypto Flow

## Goal
암호화 문서 열람과 비밀번호 입력 흐름을 정의한다.

## Design rules
- 암호화 문서 감지와 비밀번호 입력 UI는 분리한다.
- Flutter는 입력 UI와 사용자 상호작용을 담당한다.
- Rust는 암호화 감지, 복호화 시도, 오류 분류를 담당한다.
- 오류는 `wrong password`, `unsupported encryption`, `cancelled`, `invalid document`를 구분한다.
- 비밀번호는 메모리에만 유지하고 영구 저장하지 않는다.

## Initial flow
1. 사용자 문서 열기 요청
2. Rust가 암호화 여부 감지
3. 암호화 문서면 `password_required` 상태 반환
4. Flutter가 비밀번호 입력 UI 표시
5. 사용자가 비밀번호 입력 또는 취소
6. Flutter가 `submitPassword` 요청
7. Rust가 복호화 시도
8. 성공 시 문서 open success 반환
9. 실패 시 `invalid_password` 또는 `unsupported_encryption` 반환

## Error categories
- `password_required`: 비밀번호 입력 필요
- `invalid_password`: 비밀번호 틀림, 재입력 가능
- `unsupported_encryption`: 현재 엔진이 지원하지 않는 방식
- `cancelled`: 사용자가 입력을 중단함
- `invalid_document`: 손상 파일 또는 구조 오류

## Flutter UX policy
- 첫 감지 시 password dialog 표시
- 잘못된 비밀번호면 동일 dialog에서 오류 메시지와 함께 재입력 허용
- 사용자가 취소하면 open flow 전체를 취소 상태로 종료
- 비밀번호는 세션 메모리에만 유지
- remember password 기능은 MVP 제외

## Rust response policy
- 복호화 실패 원인을 코드 기반으로 반환
- 비밀번호 틀림과 지원 불가 암호화는 반드시 구분
- 취소는 UI 계층에서 발생하므로 별도 cancelled 상태를 Flutter가 관리 가능해야 함

## Deferred items
- biometric unlock integration
- secure keychain storage
- multiple password attempts telemetry
