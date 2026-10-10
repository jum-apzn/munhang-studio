# 문항공방 · Munhang Studio

**PDF와 이미지 속 수학 문항을, 내 수업에 맞는 시험지로.**

필요한 문제를 고르고, 본문과 수식을 다듬고, 번호·배점·풀이 공간을 정해 HWPX로 내보내세요. 문항공방은 수학 선생님과 튜터를 위한 **로컬 오픈소스 시험지 편집기**입니다.

[빠른 시작](#빠른-시작) · [사용 예시](#이렇게-사용하세요) · [OCR 설정](docs/OCR.md) · [기여하기](CONTRIBUTING.md)

## 28초 미리보기

https://github.com/user-attachments/assets/85115849-a2ea-4841-92de-5a503099fa0e

[전체 영상 보기](https://github.com/user-attachments/assets/85115849-a2ea-4841-92de-5a503099fa0e) · [MP4 다운로드](https://github.com/jum-apzn/munhang-studio/raw/refs/heads/main/assets/promo/munhang-studio-spacious-slider-28s.mp4) · [미리보기 이미지](assets/promo/munhang-studio-promo-poster.png)

*지원 기능을 바탕으로 사용 흐름을 재구성한 모션 영상입니다. 한국어 본문은 수동 번역했으며, 수식 전환과 슬라이더는 편집 과정을 표현한 연출입니다. [영상 출처와 제작 설명](assets/promo/credits.txt)*

## 시험지 만들기에 필요한 것들, 한곳에서

- **필요한 문항만 골라 담기** — PDF·PNG·JPEG에서 본문, 수식, 그림 영역을 직접 선택하세요.
- **다시 타이핑하는 수고 줄이기** — 로컬 한국어·영문 OCR로 본문 초안을 가져오고, 선택 설치한 수식 OCR로 수식 입력을 도와줍니다.
- **수식을 이미지 대신 수식 개체로** — 한컴 수식 스크립트를 편집하고 HWPX의 네이티브 수식 개체로 저장합니다. 수식 OCR의 LaTeX 원문도 함께 보존합니다.
- **내 수업에 맞는 구성** — 문항 순서, 시작 번호, 배점, 풀이 여백과 1단·2단 레이아웃을 조절하세요.
- **다음 수업에도 이어서** — 프로젝트 JSON을 저장해 문항을 다시 편집하고 활용할 수 있습니다.
- **문서는 내 컴퓨터에** — 가져오기부터 OCR·HWPX 생성까지 로컬에서 처리합니다. 계정 가입, API 키, 유료 API가 필요 없습니다.

## 빠른 시작

현재 **0.1 개발 프리뷰**는 소스에서 실행할 수 있습니다.

### 1. 실행 환경 준비

[Node.js](https://nodejs.org/) 22.18 이상, npm, Rust stable 1.90 이상과 [Tauri 운영체제별 필수 도구](https://v2.tauri.app/start/prerequisites/)를 준비하세요.

- Windows: Microsoft C++ Build Tools, WebView2
- macOS: Xcode command-line tools
- Linux: GTK·WebKit 개발 패키지

### 2. 데스크톱 앱 실행

```sh
git clone https://github.com/jum-apzn/munhang-studio.git
cd munhang-studio
npm ci
npm run tauri dev
```

이미지로 직접 문항을 편집하며 시작할 수 있습니다. PDF 가져오기와 OCR을 쓰려면 [로컬 도구 설치 안내](docs/OCR.md)를 따라 Poppler·Tesseract를 추가하세요. 수식 OCR은 PP-FormulaNet_plus-S를 선택 설치합니다. 초기 도구·모델 다운로드에는 인터넷과 저장 공간이 필요합니다.

### 3. 첫 시험지 만들기

1. **가져오기** — PDF 또는 PNG/JPEG를 엽니다.
2. **고르기** — 원본에서 문항·본문·수식·그림 영역을 드래그합니다.
3. **다듬기** — OCR 결과를 확인하거나 직접 입력하고, 본문과 수식을 수정합니다.
4. **구성하기** — 순서·번호·배점·풀이 여백·단 수를 설정합니다.
5. **저장하기** — 프로젝트 JSON을 보관하고 시험지를 HWPX로 내보냅니다.

브라우저에서 이미지 영역 선택·직접 편집·JSON 저장을 먼저 둘러보려면 `npm run dev`를 실행하세요. PDF·OCR·HWPX 처리는 데스크톱 앱에서 이용합니다.

## 이렇게 사용하세요

### 수업용 연습지

여러 페이지에서 오늘 배운 개념에 맞는 문항을 고르세요. 쉬운 문제부터 순서를 잡고, 서술형에는 풀이 공간을 넉넉하게 배치합니다.

### 다시 풀어 보는 복습지

저장한 프로젝트에서 필요한 문항을 남기고 시작 번호와 배점을 바꾸세요. 새 파일로 저장하면 다음 수업용 구성을 따로 관리할 수 있습니다.

### 수식과 그림을 함께 쓰는 시험지

본문은 텍스트로, 수식은 한컴 스크립트로 편집하고 도형은 원본 이미지 영역으로 담으세요.

입력할 수 있는 간단한 한컴 수식 예시:

```text
x^2 + y^2 = 1
{a+b} over {2}
sqrt {x+1}
```

OCR로 얻은 LaTeX는 지원되는 표현을 한컴 스크립트로 변환하며, 복잡한 표현은 원문을 보면서 직접 다듬을 수 있습니다.

### 명령줄로 HWPX 만들기

Rust CLI로 예제 프로젝트를 바로 내보낼 수도 있습니다.

```sh
cargo run -p munhang-cli -- export examples/sample-project.json my-worksheet.hwpx
cargo run -p munhang-cli -- status
cargo run -p munhang-cli -- ocr examples/fixtures/korean.png
# 선택 수식 모델 설치 후
cargo run -p munhang-cli -- formula examples/fixtures/formula.png
cargo run -p munhang-cli -- import-pdf examples/fixtures/one-page.pdf pages.json
```

출력에는 새 파일명을 지정하세요. 프로젝트 형식과 자동화 연동은 [JSON 계약](docs/CONTRACT.md)을 참고하세요.

## 작업 팁

PDF는 최대 20쪽, 입력 파일은 최대 40 MiB까지 다룹니다. 문항 영역은 직접 선택하며, OCR 초안을 교정한 뒤 사용하세요. 프로젝트와 함께 원본 파일을 보관하고, 배포·인쇄 전 사용하는 한컴오피스에서 글꼴과 페이지 배치를 확인하면 좋습니다. 출력 형식은 HWPX입니다.

## 개발과 기여

Rust가 문서 모델·로컬 처리·HWPX 생성을 맡고, Tauri 2와 TypeScript로 편집 화면을 구성합니다. [Ubuntu·Windows 네이티브 빌드](https://github.com/jum-apzn/munhang-studio/actions/runs/38060703532)를 CI로 확인합니다.

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
npm test
npm run build
```

- [기여 안내](CONTRIBUTING.md)
- [OCR 설치와 수식 변환](docs/OCR.md)
- [HWPX 구현](docs/HWPX-IMPLEMENTATION.md)
- [개발 검증 기록](docs/VERIFICATION.md)
- [버그와 아이디어](https://github.com/jum-apzn/munhang-studio/issues)

## 라이선스와 출처

코드는 [Apache-2.0](LICENSE)입니다. 외부 도구와 모델은 별도 설치하며 [서드파티 안내](docs/THIRD_PARTY.md)에 출처와 라이선스를 정리했습니다.

영상 문항: Siyavula Education, *Mathematics Grade 11* §2.3, Example 8 · [원문](https://www.siyavula.com/read/za/mathematics/grade-11/equations-and-inequalities/02-equations-and-inequalities-03) · [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/). 한국어 수동 번역·재조판·계수 3→5 편집을 적용했습니다. [전체 크레딧](assets/promo/credits.txt).
