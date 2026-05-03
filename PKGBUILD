# Maintainer: Said Mrini <smrini@example.com>
pkgname=shifttab
pkgver=0.1.0
pkgrel=1
pkgdesc="A Zsh TUI autocomplete tool for dynamically finding flags and arguments directly from man pages/--help outputs."
arch=('x86_64' 'i686' 'aarch64')
url="https://github.com/smrini/ShiftTab"
license=('MIT')
depends=('zsh')
makedepends=('cargo')
optdepends=('man: for parsing command man pages'
            'util-linux: for col command (ANSI stripping)'
            'tldr: for command examples in extended mode')
provides=("shifttab")
conflicts=("shifttab-git")
source=("${url}/archive/refs/tags/v${pkgver}.tar.gz")
sha256sums=('c498248ec282085fb528ce19ea6f6c4a2bc2257a2c4a6da72ace73774126c777')

build() {
    cd "$srcdir/ShiftTab-${pkgver}"
    
    # Use the optimized config in Cargo.toml
    cargo build --release --locked --manifest-path ./Cargo.toml
}

package() {
    cd "$srcdir/ShiftTab-${pkgver}"
    
    # 1. Install the executable
    install -Dm755 "target/release/ShiftTab" "$pkgdir/usr/bin/ShiftTab"
    
    # 2. Install the Zsh plugin file globally to the standard Zsh plugin directory
    install -Dm644 "shifttab.zsh" "$pkgdir/usr/share/zsh/plugins/shifttab/shifttab.zsh"
    
    # (Optional) Install a snippet explaining how to source it
    echo "# To use ShiftTab, add the one of the following at the end your ~/.zshrc:"
    echo "source /usr/share/zsh/plugins/shifttab/shifttab.zsh"
    echo "Or"
    echo 'eval "$(ShiftTab --init zsh)"'
}