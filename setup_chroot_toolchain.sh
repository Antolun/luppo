#!/usr/bin/env bash

# ==============================================================================
# PiSi Chroot Toolchain & Core Builder Orchestrator
# ==============================================================================
# Bu betik stable Chroot ortamını kurar, bootstrap toolchain paketlerini derler,
# core deposundaki 177 paketi topological sıraya göre derler ve Docker imajı üretir.
# ==============================================================================

set -e

# Renk tanımları
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "${CYAN}======================================================================${NC}"
echo -e "${GREEN}🚀 PiSi Chroot Temel Araç Takımı & Core İnşa Otomasyonu Başlatılıyor...${NC}"
echo -e "${CYAN}======================================================================${NC}"

# 1. Root Yetkisi Kontrolü
if [ "$EUID" -ne 0 ]; then
    echo -e "${RED}Hata: Bu betik sanal sistem mount işlemleri ve chroot nedeniyle ROOT yetkisi (sudo) gerektirir!${NC}"
    exit 1
fi

# 2. Core Depo Dizin Seçimi ve Doğrulanması
DEFAULT_CORE_DIR="/media/pisicik/DEPO/PISILINUX/PisiLinux_docker/core"
read -p "Core paket tarifleri dizinini girin [$DEFAULT_CORE_DIR]: " USER_CORE_DIR
CORE_DIR=${USER_CORE_DIR:-$DEFAULT_CORE_DIR}

if [ ! -d "$CORE_DIR" ]; then
    echo -e "${RED}Hata: Belirtilen dizin bulunamadı: $CORE_DIR${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Core dizini doğrulandı: $CORE_DIR${NC}\n"

# 3. pisi CLI binary'sinin derlenmesi ve yerinin belirlenmesi
echo -e "${BLUE}[1/5] pisi CLI binary'si derleniyor...${NC}"
cargo build --release
PISI_BIN="./target/release/pisi"
echo -e "${GREEN}✓ pisi başarıyla derlendi.${NC}\n"

# 4. Tariflerin yerel dizine linklenmesi
echo -e "${BLUE}[2/5] Core paket tarifleri yerel recipes dizinine aktarılıyor...${NC}"
rm -rf ./recipes
mkdir -p ./recipes
for d in "$CORE_DIR"/*/*; do
    if [ -d "$d" ] && [ -f "$d/pspec.xml" ]; then
        pkg_name=$(basename "$d")
        ln -sf "$d" "./recipes/$pkg_name"
    fi
done

echo -e "${GREEN}✓ Paket tarifleri başarıyla linklendi.${NC}\n"



# 5. Chroot Ortamının Başlatılması ve Sanal Dosya Sistemleri Mount İşlemi
echo -e "${BLUE}[3/5] /mnt/chroot dizin yapısı ve mount işlemleri başlatılıyor...${NC}"
$PISI_BIN toolchain --start
echo -e "${GREEN}✓ Chroot dizinleri ve sanal dosya sistemleri mount edildi.${NC}\n"

# Chroot altına recipes kopyala (toolchain --update'in görmesi için)
mkdir -p /mnt/chroot/recipes
cp -r ./recipes/* /mnt/chroot/recipes/ || true

# 6. Chroot Bootstrap Toolchain Paketlerinin Derlenmesi
echo -e "${BLUE}[4/5] Chroot Bootstrap Toolchain Paketleri (binutils, gcc, glibc vb.) derleniyor...${NC}"
$PISI_BIN toolchain --update
echo -e "${GREEN}✓ Chroot Bootstrap Toolchain derlemesi başarıyla tamamlandı.${NC}\n"

# 7. Core Depo Paketlerinin Topological Sırada Derlenmesi
echo -e "${BLUE}[5/5] Core deposundaki 177 paket topological sırayla inşa ediliyor...${NC}"

# Çakışmasız topological derleme sırası
core_packages=(
  "libfastjson" "keyutils" "zip" "yasm" "python-setuptools" "libgpg-error" "pcmciautils"
  "libee" "libatomic_ops" "libmd" "sysfsutils" "unifdef" "chrpath" "nss-mdns" "libidn"
  "mdadm" "unzip" "ncompress" "libestr" "python-pytz" "busybox" "dietlibc" "wireless-tools"
  "util-macros" "libunwind" "mtools" "initrd" "python-MarkupSafe" "python-Pygments"
  "lzo" "miscfiles" "procps" "gnuconfig" "libbsd" "libpipeline" "libpng" "time"
  "os-prober" "v86d" "nspr" "libaio" "linux-firmware" "duktape" "sysvinit" "liblogging"
  "libgcrypt" "less" "numactl" "libunistring" "sgml-common" "mailcap" "wireless-regdb"
  "libuv" "libsigsegv" "which" "libidn2" "python-pyliblzma" "python-six" "python-imagesize"
  "python-sphinx-alabaster-theme" "catbox" "python-Jinja2" "python-requests" "python-Babel"
  "gc" "libssh2" "mkinitramfs" "check" "json-c" "rsyslog" "bc" "rhash" "mpdecimal"
  "busybox-mkinitcpio" "libXau" "xorg-proto" "xorg-util" "xtrans" "tcp-wrappers" "gperftools"
  "slang" "psmisc" "popt" "fuse" "man-db" "mingetty" "timezone" "libcap-ng" "piksemel"
  "pycurl" "cracklib" "cython" "python-psutil" "snappy" "ca-certificates" "autoconf-archive"
  "perl-XML-Parser" "libXdmcp" "freetype" "python3-packaging" "mit-kerberos" "xcb-proto"
  "libnghttp2" "docbook-xml" "lvm2" "lsb-release" "libpcre2" "libpcre" "gobject-introspection"
  "shadow" "guile" "kmod" "gmp" "re2c" "gperf" "groff" "grub2" "biosdevname" "urlgrabber"
  "python-sphinx" "curl" "kbd" "cryptsetup" "kernel" "leveldb" "efivar" "man-pages"
  "plyvel" "intltool" "libxcb" "python3-setuptools" "docbook-xsl" "glib2" "autogen" "pciutils"
  "glpk" "isl" "mpfr" "libseccomp" "audit" "ndiswrapper" "klibc" "module-broadcom-wl"
  "module-bbswitch" "module-virtualbox" "rtl88x2bu" "module-virtualbox-guest" "pisi" "eudev"
  "libX11" "scons" "asciidoc" "xmlto" "cpupowertools" "libmpc" "disktype" "dhcpcd" "libusb"
  "fuse3" "dbus" "pisilinux-python" "mkinitcpio" "libusb-compat" "usbutils" "e2fsprogs"
  "dbus-glib" "polkit" "comar-api" "libtirpc" "dbus-python" "pypolkit" "net-tools" "libnsl"
  "pam" "vixie-cron" "mudur"
)

total_pkg=${#core_packages[@]}

for i in "${!core_packages[@]}"; do
    pkg_name="${core_packages[$i]}"
    current_num=$((i + 1))
    echo -e "${YELLOW}➔ [$current_num/$total_pkg] Derleniyor: $pkg_name...${NC}"
    
    # Emerge komutu ile Chroot chroot hedefinde (/mnt/chroot) paketi inşa et
    if $PISI_BIN emerge --target=/mnt/chroot "$pkg_name"; then
        echo -e "${GREEN}✓ Başarılı: $pkg_name${NC}"
    else
        echo -e "${RED}Hata: $pkg_name derlenirken hata oluştu! Süreç durduruluyor.${NC}"
        exit 1
    fi
done

# ==============================================================================
# Chroot Kılavuzu (stable-systemd) Uyarınca Sistem Yapılandırması (Mudur Init Uyumlu)
# ==============================================================================
echo -e "${BLUE}[5.5/5] Chroot Sistem Yapılandırması Yapılıyor (systemd yerine MUDÜR uyumlu)...${NC}"

# /etc/fstab oluşturulması (Chroot Bölüm 9.2)
cat > /mnt/chroot/etc/fstab << "EOF"
# Begin /etc/fstab

# file system  mount-point  type     options             dump  fsck
#                                                              order

/dev/sda2      /            ext4     defaults            1     1
proc           /proc        proc     nosuid,noexec,nodev 0     0
sysfs          /sys         sysfs    nosuid,noexec,nodev 0     0
devpts         /dev/pts     devpts   gid=5,mode=620      0     0
tmpfs          /run         tmpfs    defaults            0     0
devtmpfs       /dev         devtmpfs mode=0755,nosuid    0     0

# End /etc/fstab
EOF

# /etc/hostname oluşturulması (Chroot Bölüm 9.4.1)
echo "pisilinux-chroot" > /mnt/chroot/etc/hostname

# /etc/hosts oluşturulması (Chroot Bölüm 9.4.2)
cat > /mnt/chroot/etc/hosts << "EOF"
# Begin /etc/hosts

127.0.0.1 localhost pisilinux-chroot
::1       localhost ip6-localhost ip6-loopback

# End /etc/hosts
EOF

# /etc/resolv.conf oluşturulması
cat > /mnt/chroot/etc/resolv.conf << "EOF"
nameserver 8.8.8.8
nameserver 1.1.1.1
EOF

# Mudur Init Yapılandırması (/etc/conf.d)
mkdir -p /mnt/chroot/etc/conf.d
echo 'KEYMAP="trq"' > /mnt/chroot/etc/conf.d/keymap
echo 'HOSTNAME="pisilinux-chroot"' > /mnt/chroot/etc/conf.d/hostname

echo -e "${GREEN}✓ Chroot Sistem Yapılandırma dosyaları başarıyla oluşturuldu (Müdür yapılandırması tamamlandı).${NC}\n"

# 8. Docker İmajının Paketlenmesi ve Çıkarılması
echo -e "\n${BLUE}🐳 Tüm paket derlemeleri bitti. Docker imajı oluşturuluyor...${NC}"
tar -C /mnt/chroot -c . | docker import - pisi-linux-chroot:latest

echo -e "\n${GREEN}======================================================================${NC}"
echo -e "${GREEN}🎉 TEBRİKLER! Chroot Temel Araç Takımı & Core Dağıtım Derlemesi Tamamlandı!${NC}"
echo -e "${GREEN}🐳 Docker İmajı başarıyla içe aktarıldı: pisi-linux-chroot:latest${NC}"
echo -e "${GREEN}======================================================================${NC}"

