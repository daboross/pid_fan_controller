# Maintainer: thundermikey

pkgname=pid-fan-controller
pkgver=0.r9.7a94b51
pkgrel=1
pkgdesc="PID fan controller with Python3"
arch=('any')
url="https://github.com/ThunderMikey/pid_fan_controller"
license=('GPL3')
depends=('python3' 'python-simple-pid')
provides=("$pkgname")
source=('pid_fan_controller.py'
        'set_fan_control_mode.sh'
        'pid-fan-controller.service')
md5sums=('d74dd33ba0569b3022bbe57f7ccae97a'
         '6b86fe5f8bdad594136895f497575619'
         '07f2d4bba9286a9a5119a5a33265508a')

pkgver() {
  printf "0.r%s.%s" "$(git rev-list --count HEAD)" "$(git rev-parse --short HEAD)"
}

package() {
  echo $pkgdir
  install -m 644 -Dt "$pkgdir/usr/lib/systemd/system/" pid-fan-controller.service
  install -m 755 -Dt "$pkgdir/usr/share/$pkgname/" pid_fan_controller.py set_fan_control_mode.sh
}

# vim:set ts=2 sw=2 et:
