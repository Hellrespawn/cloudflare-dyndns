# Maintainer: Stef Korporaal
pkgname=cloudflare-dyndns
pkgver=0.2
pkgrel=3
pkgdesc="Dynamic DNS for CloudFlare."
arch=('any')
url="https://github.com/Hellrespawn/cloudflare-dyndns"
license=('unknown')
depends=('jq')
backup=("etc/cloudflare-dyndns/cloudflare-dyndns.conf")

source=("cloudflare-dyndns.sh"
		"service"
		"timer"
		"cloudflare-dyndns.conf")

sha256sums=('35e9201794ef7b81c2cc711c01fe27223b7bdfd5f9c0c917d9decaf18892c856'
            'f51bc911fc085cbd28e2e38bf62b4cc5886ed7a4dac2d41cf432bc373a3e897e'
            '720ebde0cd013756bf3ade0eff876ad7d503ea7a13ec00b48d960170dca65673'
            'a814ed5079a20af7ca7471a6bec2dad137ba90806e36b06c3d3fe71dfb7387cd')

package() {
	install -Dm755 "$srcdir/cloudflare-dyndns.sh" "$pkgdir/usr/bin/cloudflare-dyndns"
	install -Dm644 "$srcdir/cloudflare-dyndns.conf" "$pkgdir/etc/cloudflare-dyndns/cloudflare-dyndns.conf"
	install -Dm644 "$srcdir/service" "$pkgdir/usr/lib/systemd/system/cloudflare-dyndns.service"
	install -Dm644 "$srcdir/timer" "$pkgdir/usr/lib/systemd/system/cloudflare-dyndns.timer"
}
