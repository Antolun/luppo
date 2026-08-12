# PiSi Paketçi Kılavuzu

Bu kılavuz, LupuS/PiSiLinux için KDL formatında paket tarifleri hazırlamak, mevcut tarifleri güncellemek ve servis tanımları oluşturmak isteyen paketçiler için hazırlanmıştır.

---

## İçindekiler

1. [PiSi Paket Klasör Yapısı](#1-pisi-paket-klasör-yapısı)
2. [KDL Formatına Giriş](#2-kdl-formatına-giriş)
3. [PisiPackage Şeması](#3-pisipackage-şeması)
4. [Source Bloğu](#4-source-bloğu)
5. [Package Bloğu](#5-package-bloğu)
6. [History Bloğu](#6-history-bloğu)
7. [Actions (İnşa Adımları)](#7-actions-inşa-adımları)
8. [ActionsAPI Referansı](#8-actionsapi-referansı)
9. [COMAR Servis Tanımı](#9-comar-servis-tanımı)
10. [Bileşen ve Grup Dosyaları](#10-bileşen-ve-grup-dosyaları)
11. [Dağıtım Tanımı](#11-dağıtım-tanımı)
12. [Gerçek Paket Örnekleri](#12-gerçek-paket-örnekleri)
13. [Sık Yapılan Hatalar](#13-sık-yapılan-hatalar)

---

## 1. PiSi Paket Klasör Yapısı

Her paket, depo içinde kendi adını taşıyan bir dizinde bulunur:

```
kategori/kütüphane/paket_adi/
├── pspec.kdl              # Paket tanım dosyası (PisiPackage)
├── comar/                  # COMAR entegrasyon dosyaları
│   └── service.kdl         # Servis tanımı (opsiyonel)
│   └── pakhandler.py       # Paket yönetici betiği (opsiyonel)
│   └── package.py          # Paket betiği (opsiyonel)
├── files/                  # Yamalar ve ek dosyalar
│   └── *.patch             # Kaynak kod yamaları
│   └── *.conf              # Yapılandırma dosyaları
```

---

## 2. KDL Formatına Giriş

PiSi, paket tanımları için **KDL** (KDL Document Language) formatını kullanır. KDL, XML'e göre daha yalın ve insan okunabilir bir yapı sunar.

### Temel Kurallar

- Her düğüm (node) bir isim ve isteğe bağlı değerler/propertiler içerir
- Bloklar `{ }` ile tanımlanır
- Dizeler çift tırnak `"..."` içinde yazılır
- Yorumlar `//` ile başlar
- Property'ler `anahtar="değer"` şeklinde yazılır

### Gösterim Tipleri

KDL'de 3 farklı gösterim mümkündür (parser hepsini destekler):

```kdl
// 1. PascalCase düğüm (yeni standart, önerilen)
Name "paket-adi"

// 2. lowercase düğüm (eski)
name "paket-adi"

// 3. Property (en eski)
name="paket-adi"
```

Bu kılavuzda **PascalCase düğüm** standartı kullanılmıştır.

---

## 3. PisiPackage Şeması

```
PisiPackage {
    Source { ... }       // Kaynak paket bilgileri (1 adet)
    Package { ... }      // Çıktı paketleri (1 veya daha fazla)
    Package { ... }
    History { ... }      // Sürüm geçmişi (opsiyonel)
}
```

`PisiPackage` en üst düzey kapsayıcıdır. İçinde bir adet `Source`, bir veya daha fazla `Package` ve isteğe bağlı `History` bulunur.

---

## 4. Source Bloğu

`Source` bloğu, paketin kaynak kodu, lisansı, arşivi ve derleme bağımlılıklarını tanımlar.

```kdl
Source {
    Name "freetype"
    Homepage "https://www.freetype.org/"
    Packager {
        Name "PisiLinux Community"
        Email "admins@pisilinux.org"
    }
    License "FTL"
    License "GPLv2"
    Summary "A high-quality and portable font engine"
    Summary lang="tr" "Yüksek kaliteli ve taşınabilir yazı tipi motoru"
    Description "FreeType 2 is a software font engine..."
    Description lang="tr" "..."
    PartOf "system.base"
    Icon "pisi-software-all"
    Screenshot "https://example.com/screenshot.png"
    Provides {
        Isa "library"
    }
    Archive sha1sum="62e26b89..." type="tarxz" {
        "mirrors://sourceforge/freetype/freetype-2.14.3.tar.xz"
    }
    BuildDependencies {
        Dependency "bzip2"
        Dependency "zlib-devel"
        Dependency "libpng-devel"
    }
    Patches {
        Patch "fix-build.patch" level="1"
        Patch "security-fix.patch" level="1" compression-type="xz"
    }
    AdditionalFiles {
        AdditionalFile "extra-config.conf" target="/etc/extra.conf"
    }
}
```

### Alanlar

| Düğüm | Açıklama | Zorunlu |
|-------|----------|---------|
| `Name` | Kaynak paket adı | Evet |
| `Homepage` | Proje ana sayfası | Hayır |
| `Packager` | Paketçi bilgisi (`Name` + `Email` alt düğümleriyle) | Önerilen |
| `License` | Lisans (birden çok kullanılabilir) | Evet |
| `Summary` | Kısa açıklama (`lang` ile çeviri eklenebilir) | Evet |
| `Description` | Uzun açıklama (`lang` ile çeviri eklenebilir) | Hayır |
| `PartOf` | Bileşen adı | Hayır |
| `Icon` | Paket simgesi | Hayır |
| `Screenshot` | Ekran görüntüsü URL'si | Hayır |
| `Provides` | Sanal paket / ISA bildirimi | Hayır |
| `Archive` | Kaynak arşiv (birden çok olabilir) | Evet (en az 1) |
| `BuildDependencies` | Derleme bağımlılıkları | Hayır |
| `Patches` | Kaynak kod yamaları | Hayır |
| `AdditionalFiles` | Ek dosyalar | Hayır |
| `Architecture` | Hedef mimari | Hayır |
| `BuildFlags` | Derleme bayrakları | Hayır |

### Archive

Kaynak kod arşivinin indirileceği URL ve bütünlük bilgisi:

```kdl
// Tek arşiv
Archive sha1sum="a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6q7r8s9t0" type="tarxz" {
    "mirrors://sourceforge/proje/proje-1.0.tar.xz"
}

// Çoklu arşiv (target ile alt dizine açma)
Archive sha1sum="abc123" type="tarxz" target="proje-1.0/external/" {
    "https://example.com/ek-arsiv.tar.xz"
}

// Binary arşiv (kaynak kodu olmayan, sadece kopyalanacak)
Archive sha1sum="def456" type="binary" target="proje-1.0/external/tarballs/" {
    "https://example.com/ikili-dosya.bin"
}
```

**Archive tipleri:** `tarxz` (varsayılan), `targz`, `tarbz2`, `zip`, `binary`

### BuildDependencies

```kdl
BuildDependencies {
    Dependency "cmake"
    Dependency "gcc" version-from=">=9.3.0"
    Dependency "pkgconfig" release="current"
}
```

### Patches

```kdl
Patches {
    Patch "fix-arch.patch" level="1"
    Patch "security-fix.patch" level="1" compression-type="xz"
}
```

`compression-type` değerleri: `gz`, `xz`, `bz2`, `zst`

---

## 5. Package Bloğu

Her `Package` bloğu, kaynak koddan üretilen bir çıktı paketini tanımlar. Bir `Source` birden fazla `Package` üretebilir (ana paket, geliştirme paketi, 32-bit paket, dökümantasyon paketi vb.).

```kdl
Package {
    Name "freetype"
    Summary "Freetype font engine library"
    Summary lang="tr" "Freetype yazı tipi motoru kütüphanesi"
    Description "Detailed description..."
    Description lang="tr" "Detaylı açıklama..."
    Version "2.14.3"
    License "FTL"
    Homepage "https://www.freetype.org/"
    Icon "pisi-software-all"
    Screenshot "https://example.com/screenshot.png"
    PartOf "system.base"
    BuildType "emul32"
    BuildDependencies {
        Dependency "zlib-32bit"
    }
    RuntimeDependencies {
        Dependency "zlib" version-from=">=1.2.0"
        Dependency "bzip2"
        AnyDependency {
            Dependency "mariadb-client"
            Dependency "postgresql-client"
        }
    }
    Files {
        Path "/usr/lib" file-type="library"
        Path "/usr/share/doc" file-type="doc"
        Path "/usr/share/man" file-type="man"
        Path "/etc/freetype" file-type="config"
    }
    Actions {
        setup "./configure --prefix=/usr"
        build "make"
        install "make DESTDIR={install_root} install"
        check "make test"
        pre-install "pre-install-script.sh"
        post-install "post-install-script.sh"
        pre-remove "pre-remove-script.sh"
        post-remove "post-remove-script.sh"
        pre-upgrade "pre-upgrade-script.sh"
        post-upgrade "post-upgrade-script.sh"
        NoStrip "/lib" "/boot"
        install-filters "*.bak;*.orig"
    }
    Provides {
        Isa "app:console"
        Comar provide="System.Service" script="service.py" name="ornekd"
        Comar provide="System.Package" script="package.py"
    }
    Replaces {
        Package "eski-paket"
        Package "baska-eski-paket"
    }
    Conflicts {
        Package "cakisan-paket"
    }
    AdditionalFiles {
        AdditionalFile "ornek.conf" target="/etc/ornek.conf"
        AdditionalFile "ornek.service" target="/usr/lib/systemd/system/ornek.service"
    }
    Users {
        User "kullanici" uid=1000 gid=1000 home="/home/kullanici" shell="/bin/bash"
    }
    Groups {
        Group "grubadi" gid=1000
    }
}
```

### Alanlar

| Düğüm | Açıklama | Zorunlu |
|-------|----------|---------|
| `Name` | Paket adı | Evet |
| `Summary` | Kısa açıklama (`lang` ile çeviri) | Evet |
| `Description` | Uzun açıklama | Hayır |
| `Version` | Paket sürümü | Hayır |
| `License` | Lisans (Source'tan farklıysa) | Hayır |
| `Homepage` | Proje sayfası | Hayır |
| `Icon` | Paket simgesi | Hayır |
| `Screenshot` | Ekran görüntüsü | Hayır |
| `PartOf` | Bileşen adı | Önerilen |
| `BuildType` | Derleme türü (`emul32` vb.) | Hayır |
| `BuildDependencies` | Pakete özel derleme bağımlılıkları | Hayır |
| `RuntimeDependencies` | Çalışma zamanı bağımlılıkları | Hayır |
| `Files` | Kurulacak dosyalar | Evet |
| `Actions` | Derleme/kurulum adımları | Önerilen |
| `Provides` | COMAR ve ISA bildirimleri | Hayır |
| `Replaces` | Değiştirilen eski paketler | Hayır |
| `Conflicts` | Çakışan paketler | Hayır |
| `AdditionalFiles` | Pakete özel ek dosyalar | Hayır |
| `Users` | Oluşturulacak kullanıcılar | Hayır |
| `Groups` | Oluşturulacak gruplar | Hayır |

### Dosya Tipleri (file-type)

| Değer | Açıklama |
|-------|----------|
| `executable` | Çalıştırılabilir dosyalar (`/usr/bin`) |
| `library` | Paylaşımlı/statik kütüphaneler (`/usr/lib`) |
| `header` | Başlık dosyaları (`/usr/include`) |
| `doc` | Dökümantasyon dosyaları |
| `man` | Man sayfaları |
| `config` / `conf` | Yapılandırma dosyaları |
| `data` | Veri dosyaları |
| `info` | Info sayfaları |
| `locale` | Yerelleştirme dosyaları |

### Bağımlılıklar (Dependency)

```kdl
RuntimeDependencies {
    Dependency "glibc"                          // Basit bağımlılık
    Dependency "zlib" version-from=">=1.2.0"    // Minimum sürüm
    Dependency "freetype" release="current"     // Aynı release'e sabitle
    AnyDependency {                             // Alternatif bağımlılıklar
        Dependency "mariadb-client"
        Dependency "postgresql-client"
    }
}
```

---

## 6. History Bloğu

```kdl
History {
    Update release=2 date="2026-05-18" {
        Version "1.0.1"
        Comment "Güvenlik yamaları uygulandı."
        Name "Pisi Topluluğu"
        Email "admin@pisilinux.org"
        Type "security"
        Requires "systemRestart"
    }
    Update release=1 date="2026-03-23" {
        Version "1.0.0"
        Comment "İlk sürüm."
        Name "Pisi Topluluğu"
        Email "admin@pisilinux.org"
    }
}
```

| Alan | Açıklama |
|------|----------|
| `release` | Paket sürüm numarası (tamsayı) |
| `date` | Tarih (`YYYY-MM-DD`) |
| `Version` | Yazılım sürümü |
| `Comment` | Değişiklik açıklaması |
| `Name` | Paketçi adı |
| `Email` | Paketçi e-posta |
| `Type` | Güncelleme türü (`security`, `bugfix`, `enhancement`) |
| `Requires` | Gereksinim (`systemRestart`) |

---

## 7. Actions (İnşa Adımları)

`Actions` bloğu, paketin nasıl derleneceğini ve kurulacağını tanımlar.

```kdl
Actions {
    setup "cmake . -DCMAKE_INSTALL_PREFIX=/usr"
    build "make -j{make_jobs}"
    install "make DESTDIR={install_root} install"
    check "make test"
    pre-install "echo kurulum-oncesi"
    post-install "echo kurulum-sonrasi"
    pre-remove "echo kaldirma-oncesi"
    post-remove "echo kaldirma-sonrasi"
    pre-upgrade "echo guncelleme-oncesi"
    post-upgrade "echo guncelleme-sonrasi"
    install-filters "*.bak;*.orig"
    NoStrip "/usr/lib/debug"
}
```

### Adımlar

| Düğüm | Açıklama |
|-------|----------|
| `setup` | Derleme öncesi yapılandırma |
| `build` | Derleme komutu |
| `install` | Geçici dizine kurulum |
| `check` | Test adımı (opsiyonel) |
| `pre-install` | Paket kurulumundan önce çalışır |
| `post-install` | Paket kurulumundan sonra çalışır |
| `pre-remove` | Paket kaldırmadan önce çalışır |
| `post-remove` | Paket kaldırmadan sonra çalışır |
| `pre-upgrade` | Güncellemeden önce çalışır |
| `post-upgrade` | Güncellemeden sonra çalışır |
| `install-filters` | Kurulumda filtrelenecek dosyalar |
| `NoStrip` | Strip edilmeyecek yollar |

### Değişkenler

| Değişken | Açıklama |
|----------|----------|
| `{install_root}` | Geçici kurulum dizini |
| `{make_jobs}` | Paralel iş sayısı |
| `{srcNAME}` / `{src_name}` | Kaynak paket adı |
| `{srcVERSION}` / `{src_version}` | Kaynak sürümü |
| `{SRC_VERSION}` | Kaynak sürümü (büyük harf) |
| `{default_cflags}` | Varsayılan derleyici bayrakları |
| `{KERNEL_RELEASE}` | Çekirdek sürümü |

### Alternatif Gösterim (steps)

Eski tariflerde adımlar `steps` düğümü altında noktalı virgülle ayrılmış olarak da bulunabilir:

```kdl
Actions {
    steps "./configure --prefix=/usr"
    steps "make"
    steps "make DESTDIR={install_root} install"
    steps "pisitools.dodoc('README', 'LICENSE')"
}
```

---

## 8. ActionsAPI Referansı

ActionsAPI, KDL içinde `autotools.configure()`, `pisitools.dobin()`, `cargotools.build()` gibi fonksiyonları kullanmanızı sağlar.

### 8.1 pisitools

En sık kullanılan modül. Dosyaları work dizininden install dizinine taşır, sembolik link oluşturur.

| Kullanım | Açıklama |
|----------|----------|
| `pisitools.dobin(dosya, /bin)` | Çalıştırılabiliri `/usr/bin` (veya belirtilen) hedefe kopyalar |
| `pisitools.dodir(/usr/include/awk)` | Install dizininde klasör oluşturur |
| `pisitools.dodoc(README, ChangeLog)` | Dosyaları `/usr/share/doc/<paket>/` altına kopyalar |
| `pisitools.doexe(dosya, /etc/scripts)` | Dosyayı çalıştırılabilir olarak kopyalar |
| `pisitools.dohtml(index.html)` | HTML'leri `/usr/share/doc/<paket>/html/` altına kopyalar |
| `pisitools.doinfo(*.info)` | Info dosyalarını `/usr/share/info/` altına kopyalar |
| `pisitools.dolib(libz.a, /lib)` | Kütüphaneyi hedefe kopyalar |
| `pisitools.dolib_a(libpci.a)` | Statik (`.a`) kütüphaneyi uygun izinle kopyalar |
| `pisitools.dolib_so(libdb.so)` | Paylaşımlı (`.so`) kütüphaneyi uygun izinle kopyalar |
| `pisitools.doman(logrotate.8)` | Man sayfalarını `/usr/share/man/` altına kopyalar |
| `pisitools.domo(po/tr.po, tr, app.mo)` | PO'yu MO'ya derleyip `/usr/share/locale/...` altına kopyalar |
| `pisitools.domove(/usr/bin/passwd, /bin/)` | Install dizini içinde dosya taşır/yeniden adlandırır |
| `pisitools.dosed(Makefile, -O3, %cflags%)` | Work dizinindeki dosyada sed ile değişiklik yapar |
| `pisitools.dosbin(traceroute6)` | Sistem yöneticisi çalıştırılabilirini `/usr/sbin` altına kopyalar |
| `pisitools.dosym(gzip, /bin/gunzip)` | Install dizininde sembolik link oluşturur |
| `pisitools.insinto(/etc/, nanorc.sample, nanorc)` | Dosyayı korunmuş izinlerle hedefe kopyalar |
| `pisitools.newdoc(README, README.new)` | Doc dizinine farklı isimle kopyalar |
| `pisitools.newman(less.nro, less.1)` | Man dizinine farklı isimle kopyalar |
| `pisitools.remove(/usr/lib/bad.so)` | Install dizininden dosya siler |
| `pisitools.rename(/usr/bin/bash, bash.old)` | Install dizininde dosya adını değiştirir |
| `pisitools.removeDir(/usr/libexec)` | Install dizinindeki klasörü tüm içeriğiyle siler |
| `pisitools.installHeaders(include/*.h)` | Header dosyalarını install dizinine kopyalar |

### 8.2 autotools

Autotools (configure/make) tabanlı projeler.

| Kullanım | Açıklama |
|----------|----------|
| `autotools.configure(--enable-nls)` | `./configure` çalıştırır (standart parametreler eklenir) |
| `autotools.rawConfigure(--prefix=/usr)` | `./configure` çalıştırır (ek parametre eklemeden) |
| `autotools.make()` | `make` ile derler (`-jN` otomatik eklenir) |
| `autotools.install()` | `make install` ile DESTDIR'e kurar |
| `autotools.rawInstall(DESTDIR=%install_dir%)` | Doğrudan parametrelerle `make` kurulumu |
| `autotools.aclocal(-I m4)` | `aclocal` çalıştırır |
| `autotools.autoconf()` | `autoconf` çalıştırır |
| `autotools.autoreconf()` | `autoreconf` çalıştırır |
| `autotools.automake(--add-missing)` | `automake` çalıştırır |
| `autotools.autoheader()` | `autoheader` çalıştırır |
| `autotools.fixInfoDir()` | Info dizinini düzeltir |
| `autotools.gnuconfig_update()` | Güncel `config.sub`/`config.guess` dosyalarını kopyalar |

### 8.3 cmaketools

CMake tabanlı projeler.

| Kullanım | Açıklama |
|----------|----------|
| `cmaketools.configure()` | CMake ile yapılandırır |
| `cmaketools.configure(-DBUILD_TESTING=OFF)` | Özel parametrelerle yapılandırır |
| `cmaketools.make()` | Derler |
| `cmaketools.install()` | Kurar |

### 8.4 mesontools

Meson/Ninja tabanlı projeler.

| Kullanım | Açıklama |
|----------|----------|
| `mesontools.configure()` | `meson setup builddir` ile yapılandırır |
| `mesontools.configure(--libdir=lib)` | Özel parametrelerle yapılandırır |
| `mesontools.build()` | `ninja -C builddir` ile derler |
| `mesontools.install()` | `ninja -C builddir install` ile kurar |

### 8.5 cargotools

Rust/Cargo projeleri.

| Kullanım | Açıklama |
|----------|----------|
| `cargotools.setup()` | `cargo fetch --locked` ile bağımlılıkları indirir |
| `cargotools.build()` | `cargo build --release` ile derler |
| `cargotools.test()` | `cargo test --release` ile test eder |
| `cargotools.install()` | `cargo install --path . --root <install>/usr` ile kurar |

### 8.6 kerneltools

Linux çekirdeği ve modülleri paketleme.

| Kullanım | Açıklama |
|----------|----------|
| `kerneltools.configure()` | Çekirdek yapılandırması (`make oldconfig`) |
| `kerneltools.build()` | Çekirdeği derler |
| `kerneltools.build(debugSymbols=False)` | Debug sembolleri olmadan derler |
| `kerneltools.install()` | Çekirdeği kurar (bzImage + modüller) |
| `kerneltools.installHeaders()` | Harici modül için başlık dosyalarını kurar |
| `kerneltools.installLibcHeaders()` | Linux-Libc başlıklarını kurar |

### 8.7 python3modules

Python 3 modülleri.

| Kullanım | Açıklama |
|----------|----------|
| `python3modules.build()` | `python3 -m build --wheel --no-isolation` ile derler |
| `python3modules.install()` | Wheel veya setup.py ile kurar |
| `python3modules.compile()` | `.pyc`/`.pyo` dosyalarını temizler |

### 8.8 perlmodules

Perl modülleri.

| Kullanım | Açıklama |
|----------|----------|
| `perlmodules.configure()` | `perl Makefile.PL` ile yapılandırır |
| `perlmodules.make()` | Derler |
| `perlmodules.install()` | Kurar |
| `perlmodules.removePacklist()` | `.packlist` dosyalarını temizler |
| `perlmodules.removePodfiles()` | `.pod` dosyalarını temizler |

### 8.9 qt5 / qt6 / kde6

| Kullanım | Açıklama |
|----------|----------|
| `qt5.configure()` | Qt5 projesini yapılandırır (qmake) |
| `qt6.configure()` | Qt6 projesini yapılandırır (qmake6) |
| `kde6.configure()` | KDE6 projesini yapılandırır (cmake) |

### 8.10 Diğer Derleme Araçları

| Kullanım | Açıklama |
|----------|----------|
| `sconstools.build()` | SCons ile derler |
| `sconstools.install()` | SCons ile kurar |
| `waftools.build()` | `python3 waf build` ile derler |
| `waftools.install()` | `python3 waf install` ile kurar |
| `anttools.build()` | Ant ile derler |
| `anttools.install()` | Ant ile kurar |
| `npmtools.build()` | `npm run build` ile derler |
| `npmtools.install()` | `npm install -g --prefix <install>` ile kurar |
| `gotools.build()` | `go build -v` ile derler |
| `gotools.install()` | `go install -v` ile kurar |

### 8.11 shelltools

Sistem komutları ve dosya işlemleri. Mutlak yol ile çalışır.

| Kullanım | Açıklama |
|----------|----------|
| `shelltools.cd(build_unix)` | Çalışma dizinini değiştirir |
| `shelltools.system(./configure)` | Kabuk komutu çalıştırır |
| `shelltools.copy(kaynak, hedef)` | Dosya kopyalar |
| `shelltools.copytree(kaynak, hedef)` | Dizin ağacını kopyalar |
| `shelltools.makedirs(../build)` | Dizin oluşturur |
| `shelltools.unlink(/usr/lib/bad.so)` | Dosya siler |
| `shelltools.unlinkDir(/tmp/dir)` | Dizin ve içeriğini siler |
| `shelltools.chmod(dosya, 0644)` | Dosya iznini değiştirir |
| `shelltools.chown(dosya, root, root)` | Dosya sahipliğini değiştirir |
| `shelltools.export(WANT_AUTOCONF, 2.5)` | Ortam değişkeni tanımlar |
| `shelltools.exportFlags()` | Varsayılan derleyici bayraklarını aktarır |
| `shelltools.echo(version.h, #define V 1.0)` | Dosyaya metin yazar |
| `shelltools.touch(aclocal.m4)` | Zaman damgasını günceller |
| `shelltools.symlink(/usr/bin/gzip, /bin/gunzip)` | Sembolik link oluşturur |
| `shelltools.move(kaynak, hedef)` | Dosya/dizin taşır |

### 8.12 libtools

| Kullanım | Açıklama |
|----------|----------|
| `libtools.preplib()` | `ldconfig` çalıştırır |
| `libtools.gnuconfig_update()` | Güncel `config.sub`/`config.guess` kopyalar |
| `libtools.libtoolize(--force --copy)` | Kaynağı libtool'a hazırlar |
| `libtools.gen_usr_ldscript(libhandle.so)` | `/usr/lib/` altında sahte dinamik kütüphane betiği oluşturur |

### 8.13 get (Ortam Bilgileri)

| Çağrı | Dönüş Değeri | KDL Kısayolu |
|-------|:---|:---|
| `get.ARCH` | Mimarî (`x86_64`) | `%arch%` |
| `get.HOST` | Hedef üçlüsü | `%host%` |
| `get.CFLAGS` | C derleyici bayrakları | `%cflags%` |
| `get.CXXFLAGS` | C++ derleyici bayrakları | `%cxxflags%` |
| `get.LDFLAGS` | Bağlayıcı bayrakları | `%ldflags%` |
| `get.CC` | C derleyicisi | `%cc%` |
| `get.CXX` | C++ derleyicisi | `%cxx%` |
| `get.makeJOBS` | Paralel iş sayısı | `%make_jobs%` |
| `get.srcNAME` | Kaynak paket adı | `%src_name%` |
| `get.srcVERSION` | Kaynak sürümü | `%src_version%` |
| `get.srcRELEASE` | Kaynak release | `%src_release%` |
| `get.installDIR` | Install dizini | `%install_dir%` |
| `get.workDIR` | Work dizini | `%work_dir%` |
| `get.srcDIR` | Kaynak dizini | `%src_dir%` |
| `get.buildTYPE` | Derleme türü | `%build_type%` |
| `get.curKERNEL` | Çalışan çekirdek | `%cur_kernel%` |
| `get.curPYTHON` | Python sürümü | `%cur_python%` |
| `get.curPERL` | Perl sürümü | `%cur_perl%` |
| `get.docDIR` | Doc dizini | `%doc_dir%` |
| `get.manDIR` | Man dizini | `%man_dir%` |
| `get.infoDIR` | Info dizini | `%info_dir%` |
| `get.dataDIR` | Data dizini | `%data_dir%` |
| `get.confDIR` | Konfigürasyon dizini | `%conf_dir%` |
| `get.ENV(VAR)` | Ortam değişkeni | `%env_VAR%` |

---

## 9. COMAR Servis Tanımı

Bir paket sisteme arka planda çalışan bir servis kuruyorsa, `comar/service.kdl` dosyası ile tanımlanır:

```kdl
[service]
name = "ornek-servis"
description = "Veri senkronizasyon servisi"

dependencies = [
    "network",
    "syslog"
]

type = "simple"

[service.exec]
start = "/usr/bin/ornek-servis --daemon"
stop = "/usr/bin/ornek-servis --stop"
reload = "/usr/bin/ornek-servis --reload"

pid_file = "/run/ornek-servis.pid"

[service.behavior]
restart = "on-failure"
restart_delay_ms = 2000
```

---

## 10. Bileşen ve Grup Dosyaları

### components.kdl

Depodaki bileşen hiyerarşisini tanımlar:

```kdl
component "system" {
    local-name lang="en" "System"
    local-name lang="tr" "Sistem"
    summary lang="en" "Base system components"
    description lang="en" "Core system packages"
    group "system"
    maintainer-name "PisiLinux Community"
    maintainer-email "admins@pisilinux.org"
}

component "kernel.tools" {
    local-name lang="en" "Kernel Tools"
    summary lang="en" "Kernel tools; perf, cpupowertools"
    group "system"
    maintainer-name "PisiLinux Community"
    maintainer-email "admins@pisilinux.org"
}
```

### groups.kdl

Grupları ve simgelerini tanımlar:

```kdl
group "mate.desktop" {
    local-name lang="en" "Mate Desktop"
    local-name lang="tr" "Mate Masaüstü"
    local-name lang="de" "Mate"
    icon "preferences-desktop-wallpaper"
}
```

### component.kdl (dizin işaretleyici)

Her kategori/kütüphane dizininde bulunan basit işaretleyici:

```kdl
PISI {
    Name "kernel.drivers"
}
```

---

## 11. Dağıtım Tanımı

`distribution.kdl` dosyası depo genelindeki dağıtım bilgilerini içerir:

```kdl
distribution {
    source-name "PisiLinux"
    version "2.0"
    description lang="tr" "PisiLinux 2.0 Core Deposu"
    description lang="en" "PisiLinux 2.0 Core Repository"
    type "Core"
    binary-name "PisiLinux"
    obsoletes {
        package "glibc-32bit"
        package "isl"
    }
}
```

---

## 12. Gerçek Paket Örnekleri

### Basit Paket (disktype)

```kdl
PisiPackage {
    Source {
        Name "disktype"
        Homepage "https://disktype.sourceforge.net/"
        Packager {
            Name "PisiLinux Community"
            Email "admins@pisilinux.org"
        }
        License "MIT"
        Summary "Detect content format of a disk or disk image"
        Description "Disktype is a tool to detect the content format of a disk or disk image."
        Archive sha1sum="abc123" type="targz" {
            "mirrors://sourceforge/disktype/disktype-9.tar.gz"
        }
    }
    Package {
        Name "disktype"
        RuntimeDependencies {
            Dependency "glibc"
        }
        Files {
            Path "/usr/bin" file-type="executable"
            Path "/usr/share/man" file-type="man"
        }
        Actions {
            steps "make"
            steps "pisitools.dobin(disktype, /usr/bin)"
            steps "pisitools.doman(disktype.1)"
        }
    }
    History {
        Update release=1 date="2026-01-01" {
            Version "9"
            Comment "Initial package."
            Name "PisiLinux Community"
            Email "admins@pisilinux.org"
        }
    }
}
```

### Karmaşık Paket (çoklu alt paket, çoklu arşiv)

```kdl
PisiPackage {
    Source {
        Name "linux-firmware"
        Homepage "https://git.kernel.org/pub/scm/linux/kernel/git/firmware/linux-firmware.git"
        License "GPLv2"
        License "MIT"
        License "BSD"
        Archive sha1sum="dd685266..." type="tarxz" {
            "https://mirrors.edge.kernel.org/.../linux-firmware-20260622.tar.xz"
        }
        Archive sha1sum="abc12345..." type="tarxz" target="linux-firmware/mix" {
            "https://sourceforge.net/.../common-fw.tar.xz"
        }
    }
    Package {
        Name "linux-firmware"
        RuntimeDependencies {
            Dependency "pisi"
        }
        Files {
            Path "/lib/firmware" file-type="data"
        }
        Actions {
            steps "mkdir -p {install_root}/lib/firmware"
            steps "cp -R linux-firmware-*/* {install_root}/lib/firmware/"
        }
    }
    Package {
        Name "amd-ucode"
        Summary "AMD CPU microcode"
        Files {
            Path "/lib/firmware/amd-ucode" file-type="data"
        }
    }
    History {
        Update release=1 date="2026-06-22" {
            Version "20260622"
            Comment "Release 20260622."
            Name "PisiLinux Community"
            Email "admins@pisilinux.org"
        }
    }
}
```

### Çekirdek Modülü Paketi

```kdl
PisiPackage {
    Source {
        Name "module-broadcom-wl"
        Homepage "https://www.broadcom.com/"
        License "GPL"
        Summary "Broadcom 802.11 Linux STA driver"
        Archive sha1sum="..." type="targz" {
            "https://docs.broadcom.com/.../hybrid-v35.tar.gz"
        }
        BuildDependencies {
            Dependency "kernel-module-headers" version="6.12.94"
        }
    }
    Package {
        Name "module-broadcom-wl"
        RuntimeDependencies {
            Dependency "kernel" version="6.12.94"
        }
        Files {
            Path "/lib/modules/{KERNEL_RELEASE}/kernel/drivers/net/wireless/wl.ko" file-type="library"
        }
        Actions {
            steps "cd hybrid_wl && make -C /lib/modules/{KERNEL_RELEASE}/build M=$PWD"
            steps "mkdir -p {install_root}/lib/modules/{KERNEL_RELEASE}/kernel/drivers/net/wireless/"
            steps "cp hybrid_wl/wl.ko {install_root}/lib/modules/{KERNEL_RELEASE}/kernel/drivers/net/wireless/"
        }
    }
    History {
        Update release=1 date="2026-01-01" {
            Version "6.30.223.271"
            Comment "Initial package."
            Name "PisiLinux Community"
            Email "admins@pisilinux.org"
        }
    }
}
```

### 32-bit (emul32) Alt Paket

```kdl
Package {
    Name "freetype-32bit"
    Summary "32-bit shared libraries for freetype"
    PartOf "emul32"
    BuildType "emul32"
    BuildDependencies {
        Dependency "zlib-32bit"
        Dependency "libpng-32bit"
    }
    Actions {
        steps "./configure --libdir=/usr/lib32 --disable-static --with-harfbuzz=no"
        steps "make"
        steps "make DESTDIR={install_root} install"
    }
    RuntimeDependencies {
        Dependency "zlib-32bit"
        Dependency "libpng-32bit"
    }
    Files {
        Path "/usr/lib32" file-type="library"
    }
}
```

---

## 13. Sık Yapılan Hatalar

### `PartOf` eksikliği
Her paketin bir bileşene ait olması gerekir. Ana paket genellikle `system.base`, geliştirme paketleri `system.devel`, 32-bit paketler `emul32` bileşenine dahil edilir.

### Dosya yollarında glob kullanımı
`Path` içinde `*` joker karakteri kullanılabilir: `/usr/lib/libfoo.so*`, `/usr/share/man/man8/*pv*`

### Bağımlılık sürüm constraint yazımı
```kdl
// Doğru:
Dependency "glibc" version-from=">=2.30"

// Yanlış (parser hatası):
Dependency "glibc >=2.30"
```

### COMAR yazım farkı
`Comar` (karışık harf) ve `COMAR` (tamamen büyük) parser tarafından eşdeğer kabul edilir:
```kdl
Provides {
    COMAR "System.Package" script="package.py"
    // veya:
    Comar provide="System.Package" script="package.py"
}
```

### `Screenshot` yazım farkı
`Screenshot` ve `ScreenShot` her iki yazım da geçerlidir:
```kdl
Screenshot "https://example.com/screen.png"
ScreenShot "https://example.com/screen.png"
```

### Binary arşivler
Kaynak kodu içermeyen (sadece kopyalanacak) dosyalar için `type="binary"` kullanın. Bu arşivler açılmaz, doğrudan work dizinine kopyalanır.

### Derleme adımlarında tırnak kullanımı
KDL içinde tek tırnak kullanmak isterseniz, çift tırnaklı dize içinde kullanabilirsiniz:
```kdl
steps "autoreconf '-fi'"
```

---

Bu kılavuzdaki standartlara uyarak LupuS/PiSiLinux için temiz, kararlı ve modern paketler hazırlayabilirsiniz.
