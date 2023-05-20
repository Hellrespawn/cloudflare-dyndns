# Maintainer: Your Name <youremail@domain.com>
pkgname=vimexx-dyndns
pkgver=0.1
pkgrel=1
pkgdesc="Dynamic DNS for vimexx."
arch=('any')
url="https://github.com/Hellrespawn/vimexx-dyndns"
license=('unknown')
depends=('jq')
backup=("etc/vimexx-dyndns/last-ip"
		"etc/vimexx-dyndns/vimexx-dyndns.conf"
		"etc/vimexx-dyndns/token.json")

source=("vimexx-dyndns.sh"
		"service"
		"timer"
		".env.example")

sha256sums=('393595dcccb0e43e7d8677f1f49e12111e6782d04647adea1ee6b9b662409ea6'
            '7f133c891280010989ab0613ad2139b3dba0cda25ee22e7ca38b6a20fd689410'
            '34b3de33955b5f30be3095cec8f7d5d7fb17466664b58f77f0dfbe80e1575b73'
            '702d3acfa408d2f9c12792149172aa84a673545b9a2adad3137c310e2fb39478')

package() {
	install -Dm755 "$srcdir/vimexx-dyndns.sh" "$pkgdir/usr/bin/vimexx-dyndns"
	install -Dm644 "$srcdir/service" "$pkgdir/usr/lib/systemd/system/vimexx-dyndns.service"
	install -Dm644 "$srcdir/timer" "$pkgdir/usr/lib/systemd/system/vimexx-dyndns.timer"
	install -Dm644 "$srcdir/.env.example" "$pkgdir/etc/vimexx-dyndns/vimexx-dyndns.conf"
}
