# 검증 상태 · 2026-10-10

Linux x86_64 / Rust 1.99.0에서 수행했습니다. 개발 프리뷰이며 출시용 완성본으로 인증하지 않습니다.

## 통과

- Rust 단위 테스트 24개: HWPX ZIP/XML, 스타일/이미지 참조, 수식 스크립트 보존, 1/2단, 그림 비율, XML 이스케이프, 입력/시간 제한, 보수적 LaTeX 변환, 편집 메타데이터 왕복 및 출력 중복 방지.
- cargo clippy --workspace --all-targets -- -D warnings: 경고 없음.
- 프런트엔드 유틸 14개 + DOM 상호작용 16개: 총 30개. TypeScript 정적 검사, Vite 프로덕션 빌드, Prettier 검사 통과.
- 실제 Tesseract: 합성 한국어/영문을 정확히 읽음. 실제 Poppler: 1쪽 PDF 가져오기, 21쪽 PDF 제한 오류 확인.
- 실제 PP-FormulaNet_plus-S CPU: x²+2x+1=(x+1)²를 정확한 LaTeX로 읽음. 신규 프로세스 실행 6.42초, 최대 메모리 약 806 MiB. 일반 문서 정확도 벤치마크가 아님.
- 실제 Rust CLI: 수식 이미지 → 로컬 모델 → LaTeX → 한컴 스크립트 → 원본 메타데이터 포함 JSON → HWPX 생성 경로 통과.
- 독립 python-hwpx 6.8.0 검사: 패키지 검사, 알려진 Hancom 2011 차이를 허용하는 스키마 lint, 파서 재열기 검사에서 오류 0. 원시 2024 XSD 편차는 예제 validation-report.json에 별도 기록. 완전한 XSD 준수 인증이 아님.
- 공식 한국어 모델 및 수식 가중치 다운로드는 고정 revision/체크섬 검증. 문서가 외부로 전송되는 API 코드는 없음.

## 미실행 / 환경 차단

- Tauri 네이티브 빌드: glib/GTK/WebKit 개발 패키지 부재. 공식 apt 설치 시 설정 읽기/lock 권한 오류. cargo check는 glib-sys/gobject-sys 단계에서 실패했고 앱 타입 검사까지 도달하지 못함. 권한 우회 없음.
- 실제 브라우저/WebView의 렌더링·마우스·OS 저장창: socket 권한 오류로 실행 불가. DOM 테스트는 모의 canvas/IPC를 사용하므로 실제 네이티브 검증과 다름.
- 한컴오피스에서 열기, 수식 편집, 재저장, 인쇄/페이지 나눔: 한컴 실행 환경 부재로 미실행. 독립 파서 검증으로 대신 완료 처리하지 않음.
- Windows/macOS OCR 설치 및 앱 실행: 미실행. 소스 내 CI가 Ubuntu/Windows 빌드를 시도하지만 게시 전에는 실행 결과 없음.

## 미구현

- 자동 문항 분할, 손글씨 정확도 보장, 복잡한 도형을 편집 개체로 복원, 전체 LaTeX/행렬 지원, HWP 바이너리 출력, 전체 원본 PDF 프로젝트 내 보존.
- 본문용 Paddle Korean 모델 통합: 검토만 수행. 본문은 실제 검증한 Tesseract를 사용함.
- 설치형 배포물에 Python/Paddle/Poppler/Tesseract 자동 번들링. 선택 설치는 사용자가 안내에 따라 수행해야 함.

## 재현/증거

README 명령과 docs/OCR.md를 사용하세요. verification/에는 최종 단위/통합 테스트 로그와 수식 결과 및 독립 검증 JSON을 둡니다. 원본은 모두 자체 합성 자료입니다. HWPX의 실제 한컴 수용 검사 절차는 HWPX-IMPLEMENTATION.md, UI 범위는 UI-VERIFICATION.md에 있습니다.

독립 검토 후 추가 회귀 검증: 중괄호 없는 첨자 16,000개 및 sqrt/frac 명령 1,000개 연쇄가 stack overflow 없이 제한 경고를 반환함. 수식 파서 모든 재귀 경로에 공통 깊이 제한을 적용했습니다. verification/formula-recursion-regression.json 참고.

저장 복구 회귀 검증: 100블록·500문항 한도는 추가 전에 거부, 제목/본문/수식 길이와 최종 JSON80MiB 제한은 변경 전에 확인하거나 마지막 유효 상태로 복구합니다. 손상/구버전 초과 데이터는 원문을 보존하고 자동 저장을 중단하며 복구 원문 저장 기능을 제공합니다. 독립 DOM 재검증에서 100→101블록 거부/재열기 및 손상JSON/101블록JSON 원문 불변을 확인했습니다.
