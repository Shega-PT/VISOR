"""
Build script para compilar a library Rust (Protocolo TLV) antes do PlatformIO.

Este script é executado automaticamente pelo PlatformIO antes da compilação
do firmware C/C++. Compila o crate Rust como library estática (.a) que é
posteriormente linkada pelo linker do ESP-IDF.
"""

Import("env")
import subprocess
import os
import sys

def build_rust_library(source, target, env):
    """Compila o crate Rust como library estática."""
    project_dir = env["PROJECT_DIR"]
    rust_dir = os.path.join(project_dir, "rust")

    if not os.path.exists(os.path.join(rust_dir, "Cargo.toml")):
        print("[build_rust.py] AVISO: Cargo.toml não encontrado, saltando build Rust")
        return

    print("[build_rust.py] A compilar library Rust (Protocolo TLV)...")

    # Determinar target ESP32
    esp_idf_path = os.environ.get("IDF_PATH", "")
    if esp_idf_path:
        env_idf = os.path.join(esp_idf_path, "export.sh")
    else:
        env_idf = None

    # Comando de build
    cmd = ["cargo", "build", "--release"]

    # Configurar target se toolchain esp estiver disponível
    rustup_home = os.environ.get("RUSTUP_HOME", os.path.expanduser("~/.rustup"))
    esp_toolchain = os.path.join(rustup_home, "toolchains", "esp", "bin")
    if os.path.exists(esp_toolchain):
        # Usar toolchain esp para Xtensa
        cargo_target = "xtensa-esp32-espidf"
        env_build = os.environ.copy()
        env_build["CARGO_BUILD_TARGET"] = cargo_target

        print(f"[build_rust.py] Target: {cargo_target}")
        result = subprocess.run(
            cmd,
            cwd=rust_dir,
            env=env_build,
            capture_output=True,
            text=True
        )
    else:
        # Fallback: compilar para target nativo (testes)
        print("[build_rust.py] Toolchain ESP não encontrada, compilando para host")
        result = subprocess.run(
            cmd,
            cwd=rust_dir,
            capture_output=True,
            text=True
        )

    if result.returncode != 0:
        print(f"[build_rust.py] ERRO na compilação Rust:")
        print(result.stderr)
        # Em modo teste, não falhar o build
        if "test" in env.get("BUILD_TYPE", "").lower():
            print("[build_rust.py] Modo teste — a ignorar erro Rust")
        else:
            sys.exit(1)
    else:
        print("[build_rust.py] Library Rust compilada com sucesso")

    # Copiar .a para lib/protocol_ffi/lib/
    lib_dir = os.path.join(project_dir, "lib", "protocol_ffi", "lib")
    os.makedirs(lib_dir, exist_ok=True)

    # Procurar o ficheiro .a compilado
    possible_paths = [
        os.path.join(rust_dir, "target", "xtensa-esp32-espidf", "release", "libvisor_protocol.a"),
        os.path.join(rust_dir, "target", "release", "libvisor_protocol.a"),
    ]

    for src_path in possible_paths:
        if os.path.exists(src_path):
            import shutil
            dst_path = os.path.join(lib_dir, "libvisor_protocol.a")
            shutil.copy2(src_path, dst_path)
            print(f"[build_rust.py] Library copiada para: {dst_path}")
            return

    print("[build_rust.py] AVISO: Ficheiro .a não encontrado")

# Registar hook de pré-build
env.AddPreAction("$BUILD_DIR/firmware.elf", build_rust_library)
