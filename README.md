# 문항공방 · Munhang Studio

로컬 수학 시험지 편집기. Rust가 문서 모델·HWPX 생성·로컬 처리 작업을 맡고, Tauri 2와 TypeScript UI로 문항을 편집합니다.

**0.1 개발 프리뷰**입니다. 원본 이미지 영역 선택, 본문 수정, 수식 스크립트 수정, 문항 재배열, 배점·풀이 여백, 1단/2단 HWPX 생성을 구현했습니다. 생성 HWPX의 ZIP/XML·수식 개체·이미지 참조를 테스트했지만, 한컴오피스에서 열기·인쇄·수식 편집 후 재저장 검증은 아직 하지 못했습니다. 배포용 설치 프로그램이 포함된 완성 릴리스가 아닙니다.

## 개인정보와 비용

- 문서를 서버에 올리지 않습니다. API 키·계정·유료 API를 사용하지 않습니다.
- PDF/이미지, OCR, HWPX 처리는 내 컴퓨터에서 수행합니다.
- 초기 개발 도구·선택 OCR 모델 설치에는 인터넷과 저장 공간이 필요합니다.
- 이미지로 남긴 도형은 수학 도형 편집 개체로 변환되지 않습니다.

## 빠른 실행

개발 환경: Node.js 22.18 이상, npm, Rust stable 1.90 이상. Tauri의 OS별 필수 도구도 설치해야 합니다: https://v2.tauri.app/start/prerequisites/

```sh
npm ci
npm run tauri dev
```

Windows는 Microsoft C++ Build Tools 및 WebView2, macOS는 Xcode command-line tools, Linux는 공식 안내의 GTK/WebKit 개발 패키지가 필요합니다. PDF·OCR은 추가 로컬 도구가 있을 때 활성화됩니다. 설치법: [docs/OCR.md](docs/OCR.md).

프런트엔드만 보기:

```sh
npm run dev
```

브라우저 모드에서는 이미지 영역 선택·직접 편집·JSON 저장을 볼 수 있습니다. Rust 연결이 필요한 PDF/OCR/HWPX 기능은 데스크톱 앱에서 실행하세요. 브라우저 모드는 HWPX를 만든 것처럼 성공 표시하지 않습니다.

## 사용 순서

1. PDF 또는 PNG/JPEG를 가져옵니다. PDF는 최대 20쪽, 파일은 최대 40 MB입니다.
2. 원본 페이지에서 문항·본문·수식·그림 영역을 드래그합니다.
3. 본문은 로컬 OCR으로 읽거나 직접 입력합니다. 글씨가 작거나 흐리면 인식 결과를 꼭 검토하세요.
4. 수식 OCR은 선택 설치한 공개 PP-FormulaNet_plus-S 모델을 CPU에서 실행합니다. LaTeX 원문을 보존하고 지원되는 기호는 한컴 수식으로 변환합니다. 설치·변환 한계는 OCR 문서를 확인하세요.
5. 순서·시작 번호·배점·풀이 여백·단 수를 설정합니다.
6. 프로젝트 JSON을 저장하고 HWPX로 내보냅니다. 그림은 선택한 원본 영역 이미지로 들어갑니다.

간단한 한컴 수식: `x^2 + y^2 = 1`, `{a+b} over {2}`, `sqrt {x+1}`. LaTeX와 한컴 스크립트는 동일하지 않습니다. 지원되지 않는 자동 변환은 원문과 경고를 보여 주고 직접 확인하도록 합니다.

## Rust CLI

데스크톱 UI 없이도 실제 HWPX 파일을 만들 수 있습니다.

```sh
cargo run -p munhang-cli -- export examples/sample-project.json my-worksheet.hwpx
cargo run -p munhang-cli -- status
cargo run -p munhang-cli -- ocr examples/fixtures/korean.png
# 선택 수식 모델 설치 후:
cargo run -p munhang-cli -- formula examples/fixtures/formula.png
cargo run -p munhang-cli -- import-pdf examples/fixtures/one-page.pdf pages.json
```

CLI는 기존 출력 파일을 덮어쓰지 않습니다. 새 파일명을 지정하세요. 프로젝트는 버전 1 JSON이며 [docs/CONTRACT.md](docs/CONTRACT.md)를 참고하세요.

## 검증

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
npm test
npm run build
# 로컬 Tesseract/Poppler와 한국어 모델이 설치된 경우:
MUNHANG_FIXTURES="$PWD/examples/fixtures" cargo test --workspace local_engine_smoke_test -- --ignored
```

원본 예제와 테스트 자료는 직접 합성했습니다. 최종 실행 결과와 미검증 단계는 [docs/VERIFICATION.md](docs/VERIFICATION.md)에 기록합니다.

## 범위와 한계

- 자동 문항 분할·완전한 문서 레이아웃 복원은 없습니다. 영역은 직접 고릅니다.
- 본문 OCR은 오인식이 있으며 교정이 필요합니다. 수식과 도형을 일반 문자 OCR만으로 정확히 읽는다고 주장하지 않습니다.
- 모든 LaTeX 명령/행렬/복잡한 맞춤 배열을 한컴 수식으로 변환하지 않습니다.
- 화면의 시험지 미리보기는 근사 배치입니다. 한글의 글꼴·줄바꿈·페이지 나눔과 일치하지 않을 수 있습니다.
- HWP(구형 바이너리) 출력은 없습니다. HWPX의 한컴오피스 버전별 완전 호환은 보장하지 않습니다.
- 원본 전체 PDF를 프로젝트에 저장하지 않습니다. 필요한 그림/문항 영역은 포함되며 원본 파일은 보존하세요.

## 라이선스

Apache-2.0. [THIRD_PARTY.md](docs/THIRD_PARTY.md)와 별도 설치 도구의 라이선스를 확인하세요. Poppler 실행 파일·OCR 모델은 이 소스에 포함하지 않습니다.
