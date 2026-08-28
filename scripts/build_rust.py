"""
Build script para compilar a library Rust (Protocolo TLV) antes do PlatformIO.

Compila o crate Rust como library estática (.a) para Xtensa ESP32,
utilizando a toolchain esp e -Zbuild-std para construir a std lib.
"""

Import("env")
import subprocess
import os
import sys
import shutil

def build_rust_library(source, target, env):
    """Compila o crate Rust como library estática para Xtensa ESP32."""
    project_dir = env["PROJECT_DIR"]
    rust_dir = os.path.join(project_dir, "rust")

    if not os.path.exists(os.path.join(rust_dir, "Cargo.toml")):
        print("[build_rust.py] AVISO: Cargo.toml não encontrado, saltando build Rust")
        return

    print("[build_rust.py] A compilar library Rust (Protocolo TLV)...")

    # Detectar toolchain esp
    rustup_home = os.environ.get("RUSTUP_HOME", os.path.expanduser("~/.rustup"))
    esp_toolchain = os.path.join(rustup_home, "toolchains", "esp", "bin")
    esp_toolchain_exists = os.path.exists(esp_toolchain)

    if not esp_toolchain_exists:
        print("[build_rust.py] ERRO: Toolchain ESP não encontrada em:", esp_toolchain)
        print("[build_rust.py] Instale com: cargo install espup && espup install")
        sys.exit(1)

    # Detectar IDF_PATH — PlatformIO ou variável de ambiente
    idf_path = os.environ.get("IDF_PATH", "")
    if not idf_path:
        # Procurar no PlatformIO
        pio_idf = os.path.expanduser("~/.platformio/packages/framework-espidf")
        if os.path.exists(pio_idf):
            idf_path = pio_idf
            print(f"[build_rust.py] IDF_PATH auto-detectado: {idf_path}")

    # Configurar ambiente de build
    env_build = os.environ.copy()
    env_build["RUSTUP_TOOLCHAIN"] = "esp"
    env_build["CARGO_BUILD_TARGET"] = "xtensa-esp32-espidf"
    if idf_path:
        env_build["IDF_PATH"] = idf_path

    cargo_target = env_build["CARGO_BUILD_TARGET"]
    print(f"[build_rust.py] Target: {cargo_target}")
    print(f"[build_rust.py] IDF_PATH: {idf_path or '(não definido)'}")

    # Comando de build com -Zbuild-std para construir std lib
    cmd = [
        "cargo", "build", "--release",
        "-Zbuild-std=std,panic_abort",
    ]

    print(f"[build_rust.py] Comando: {' '.join(cmd)}")
    result = subprocess.run(
        cmd,
        cwd=rust_dir,
        env=env_build,
        capture_output=True,
        text=True
    )

    if result.returncode != 0:
        print(f"[build_rust.py] ERRO na compilação Rust:")
        if result.stdout:
            print(result.stdout)
        if result.stderr:
            print(result.stderr)
        sys.exit(1)

    print("[build_rust.py] Library Rust compilada com sucesso")

    # Procurar o ficheiro .a compilado
    src_path = os.path.join(rust_dir, "target", cargo_target, "release", "libvisor_protocol.a")
    if not os.path.exists(src_path):
        # Fallback
        src_path = os.path.join(rust_dir, "target", "release", "libvisor_protocol.a")

    if not os.path.exists(src_path):
        print(f"[build_rust.py] ERRO: Ficheiro .a não encontrado")
        sys.exit(1)

    # Copiar .a para lib/protocol_ffi/lib/ (CMakeLists.txt procura aqui)
    lib_dir = os.path.join(project_dir, "lib", "protocol_ffi", "lib")
    os.makedirs(lib_dir, exist_ok=True)
    dst_path = os.path.join(lib_dir, "libvisor_protocol.a")
    shutil.copy2(src_path, dst_path)
    print(f"[build_rust.py] Library copiada para: {dst_path}")

# Registar hook de pré-build
env.AddPreAction("$BUILD_DIR/firmware.elf", build_rust_library)
