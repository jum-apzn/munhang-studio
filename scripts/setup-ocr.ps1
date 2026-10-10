param([switch]$DownloadKorean, [string]$DataDir = (Join-Path $env:LOCALAPPDATA 'MunhangStudio\tessdata'))
$ErrorActionPreference = 'Stop'
$Revision = '87416418657359cb625c412a48b6e1d6d41c29bd'
$ExpectedHash = '6b85e11d9bbf07863b97b3523b1b112844c43e713df8b66418a081fd1060b3b2'
if ($DownloadKorean) {
  New-Item -ItemType Directory -Force -Path $DataDir | Out-Null
  $Temporary = Join-Path $DataDir ([Guid]::NewGuid().ToString() + '.traineddata')
  try {
    Invoke-WebRequest -Uri "https://raw.githubusercontent.com/tesseract-ocr/tessdata_fast/$Revision/kor.traineddata" -OutFile $Temporary
    if ((Get-FileHash $Temporary -Algorithm SHA256).Hash.ToLowerInvariant() -ne $ExpectedHash) { throw 'Model checksum mismatch.' }
    Invoke-WebRequest -Uri "https://raw.githubusercontent.com/tesseract-ocr/tessdata_fast/$Revision/LICENSE" -OutFile (Join-Path $DataDir 'LICENSE-tessdata_fast.txt')
    Move-Item -Force $Temporary (Join-Path $DataDir 'kor.traineddata')
  } finally { if (Test-Path $Temporary) { Remove-Item $Temporary } }
  $env:TESSDATA_PREFIX = $DataDir
  Write-Host "Korean data installed at $DataDir. TESSDATA_PREFIX is set only in this PowerShell session. Launch the app here. Copy existing eng.traineddata here for kor+eng."
}
if (Get-Command pdftoppm -ErrorAction SilentlyContinue) { & pdftoppm -v } else { Write-Warning 'pdftoppm missing. See docs/OCR.md.' }
if (Get-Command tesseract -ErrorAction SilentlyContinue) { & tesseract --list-langs } else { Write-Warning 'Tesseract missing. See docs/OCR.md.' }
