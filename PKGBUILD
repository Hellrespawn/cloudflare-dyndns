# Maintainer: Your Name <youremail@domain.com>
pkgname=cloudflare-dyndns
pkgver=0.1
pkgrel=1
pkgdesc="Dynamic DNS for CloudFlare."
arch=('any')
url="https://github.com/Hellrespawn/cloudflare-dyndns"
license=('unknown')
depends=('jq')
backup=("etc/cloudflare-dyndns/cloudflare-dyndns.conf")

source=("cloudflare-dyndns.sh"
		"service"
		"timer"
		".env.example")

sha256sums=('87abf101d3b7ef0c7e287c704ab835ef276b54f6b2649fea271520c53d1b0453'
            '05b9f23b68f5be788174d4727e9eb9498a4c36696813b641214c98419a86dd7a'
            '720ebde0cd013756bf3ade0eff876ad7d503ea7a13ec00b48d960170dca65673'
            'a814ed5079a20af7ca7471a6bec2dad137ba90806e36b06c3d3fe71dfb7387cd')

package() {
	install -Dm755 "$srcdir/cloudflare-dyndns.sh" "$pkgdir/usr/bin/cloudflare-dyndns"
	install -Dm644 "$srcdir/service" "$pkgdir/usr/lib/systemd/system/cloudflare-dyndns.service"
	install -Dm644 "$srcdir/timer" "$pkgdir/usr/lib/systemd/system/cloudflare-dyndns.timer"
	install -Dm644 "$srcdir/.env.example" "$pkgdir/etc/cloudflare-dyndns/cloudflare-dyndns.conf"
}
