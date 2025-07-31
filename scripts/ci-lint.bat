@echo off
setlocal enabledelayedexpansion

echo 🔍 Running Spec-to-Proof CI Lint Checks...

REM Check if Bazel is available
where bazel >nul 2>&1
if %errorlevel% neq 0 (
    echo ❌ Bazel is not installed. Please install Bazel first.
    exit /b 1
)

REM 1. Bazel build check
echo 📦 Running Bazel build check...
bazel build //...
if %errorlevel% neq 0 (
    echo ❌ Bazel build failed
    exit /b 1
)
echo ✅ Bazel build successful

REM 2. Bazel test check
echo 🧪 Running Bazel tests...
bazel test //...
if %errorlevel% neq 0 (
    echo ❌ Bazel tests failed
    exit /b 1
)
echo ✅ Bazel tests passed

REM 3. TypeScript/JavaScript linting
echo 🔧 Running TypeScript/JavaScript linting...
where npx >nul 2>&1
if %errorlevel% equ 0 (
    REM Check for TypeScript files
    dir /s /b *.ts *.tsx *.js *.jsx >nul 2>&1
    if %errorlevel% equ 0 (
        REM Prettier check
        npx prettier --check "**/*.{ts,tsx,js,jsx,json,md}"
        if %errorlevel% neq 0 (
            echo ❌ Prettier formatting check failed
            exit /b 1
        )
        echo ✅ Prettier formatting check passed

        REM ESLint check
        npx eslint "**/*.{ts,tsx,js,jsx}"
        if %errorlevel% neq 0 (
            echo ❌ ESLint check failed
            exit /b 1
        )
        echo ✅ ESLint check passed
    ) else (
        echo ⚠️  No TypeScript/JavaScript files found
    )
) else (
    echo ⚠️  npx not available, skipping TypeScript/JavaScript linting
)

REM 4. Rust linting with Clippy
echo 🦀 Running Rust Clippy checks...
where cargo >nul 2>&1
if %errorlevel% equ 0 (
    for /r %%f in (Cargo.toml) do (
        echo Checking Rust crate in: %%~dpf
        pushd %%~dpf
        cargo clippy -- -D warnings
        if !errorlevel! neq 0 (
            echo ❌ Clippy failed for %%~dpf
            popd
            exit /b 1
        )
        echo ✅ Clippy passed for %%~dpf
        popd
    )
) else (
    echo ⚠️  cargo not available, skipping Rust linting
)

REM 5. Lean linting
echo 📐 Running Lean linting...
where lean >nul 2>&1
if %errorlevel% equ 0 (
    dir /s /b *.lean >nul 2>&1
    if %errorlevel% equ 0 (
        where leanlint >nul 2>&1
        if %errorlevel% equ 0 (
            leanlint **/*.lean
            if %errorlevel% neq 0 (
                echo ❌ Lean linting failed
                exit /b 1
            )
            echo ✅ Lean linting passed
        ) else (
            echo ⚠️  leanlint not available, skipping Lean linting
        )
    ) else (
        echo ⚠️  No Lean files found
    )
) else (
    echo ⚠️  lean not available, skipping Lean linting
)

REM 6. Terraform linting
echo 🏗️  Running Terraform linting...
where terraform >nul 2>&1
if %errorlevel% equ 0 (
    dir /s /b *.tf >nul 2>&1
    if %errorlevel% equ 0 (
        for /r %%f in (*.tf) do (
            echo Checking Terraform in: %%~dpf
            pushd %%~dpf
            terraform fmt -check
            if !errorlevel! neq 0 (
                echo ❌ Terraform formatting check failed for %%~dpf
                popd
                exit /b 1
            )
            echo ✅ Terraform formatting check passed for %%~dpf
            
            terraform validate
            if !errorlevel! neq 0 (
                echo ❌ Terraform validation failed for %%~dpf
                popd
                exit /b 1
            )
            echo ✅ Terraform validation passed for %%~dpf
            popd
        )
    ) else (
        echo ⚠️  No Terraform files found
    )
) else (
    echo ⚠️  terraform not available, skipping Terraform linting
)

REM 7. Protobuf linting
echo 📋 Running Protobuf linting...
where buf >nul 2>&1
if %errorlevel% equ 0 (
    if exist buf.yaml (
        buf lint
        if %errorlevel% neq 0 (
            echo ❌ Protobuf linting failed
            exit /b 1
        )
        echo ✅ Protobuf linting passed
        
        buf breaking --against '.git#branch=main'
        if %errorlevel% neq 0 (
            echo ❌ Protobuf breaking change check failed
            exit /b 1
        )
        echo ✅ Protobuf breaking change check passed
    ) else (
        echo ⚠️  No buf.yaml or buf.work.yaml found
    )
) else (
    echo ⚠️  buf not available, skipping Protobuf linting
)

REM 8. Security scanning
echo 🔒 Running security scans...
where cargo-audit >nul 2>&1
if %errorlevel% equ 0 (
    cargo audit
    if %errorlevel% neq 0 (
        echo ❌ Cargo audit failed
        exit /b 1
    )
    echo ✅ Cargo audit passed
) else (
    echo ⚠️  cargo-audit not available, skipping security scan
)

REM 9. License compliance
echo 📄 Checking license compliance...
where license-checker >nul 2>&1
if %errorlevel% equ 0 (
    license-checker --summary
    if %errorlevel% neq 0 (
        echo ❌ License compliance check failed
        exit /b 1
    )
    echo ✅ License compliance check passed
) else (
    echo ⚠️  license-checker not available, skipping license compliance check
)

echo.
echo 🎉 All lint checks passed!
echo Spec-to-Proof codebase is clean and ready for production. 