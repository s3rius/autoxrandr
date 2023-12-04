# Maintainer: s3rius <win10@list.ru>
pkgname=autoxrandr
pkgver=0.1.0
pkgrel=1
pkgdesc='Automatic xrandr configuration based on connected devices'
url="https://github.com/s3rius/autoxrandr"
arch=('i686' 'pentium4' 'x86_64' 'arm' 'armv7h' 'armv6h' 'aarch64')
makedepends=('cargo')
depends=('xorg-xrandr')
license=('MIT')

build() {
  cd "$startdir/"
  cargo build --release
}

package() {
  install -Dm 755 "$startdir/target/release/autoxrandr" -t "$pkgdir/usr/bin"
}
