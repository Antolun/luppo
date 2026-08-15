# Luppo vs. Diğer Paket Yöneticileri Karşılaştırma Analizi

Bu dosya, Rust ile yeniden yazılan **luppo** projesinin mevcut yeteneklerini, orijinal Python tabanlı Luppo ve modern Linux dağıtımlarında kullanılan diğer popüler paket yöneticileriyle karşılaştırmaktadır.

## Paket Yöneticileri Karşılaştırma Tablosu

| Özellik / Yetenek                        | luppo (Rust Sürümü)                                           | Orijinal Luppo (Python)                   | pacman (Arch Linux)                           | APT / dpkg (Debian/Ubuntu)                 | DNF / RPM (Fedora/RHEL)                      | APK (Alpine Linux)                         | XBPS (Void Linux)                              |
| :--------------------------------------- | :------------------------------------------------------------ | :---------------------------------------- | :-------------------------------------------- | :----------------------------------------- | :------------------------------------------- | :----------------------------------------- | :--------------------------------------------- |
| **Geliştirme Dili**                      | Rust                                                          | Python 2.7                                | C                                             | C / C++ / Perl                             | Python (DNF) / C (RPM)                       | C                                          | C                                              |
| **Veritabanı Altyapısı**                 | Sled (Gelişmiş Key-Value)                                     | BerkeleyDB / XML                          | Düz Dosyalar (Flat-files)                     | Düz Dosyalar (`/var/lib/dpkg/status`)      | SQLite (DNF) / BerkeleyDB (RPM)              | Düz Dosyalar / DB                          | Plist Dosyaları                                |
| **Paket Tanımlama/İnşa Formatı**         | KDL (`paket.kdl`), JSON, XML (`lopec.xml`)                    | XML (`lopec.xml`) + Python (`actions.py`) | Bash (`PKGBUILD`)                             | Debian Control + Makefile (`debian/rules`) | RPM Spec Dosyası                             | Kabuk Betiği (`APKBUILD`)                  | Kabuk Betiği (`template`)                      |
| **İkili Paket Formatı**                  | `.luppo` (ZIP + KDL/XML)                                      | `.luppo` (ZIP + XML)                      | `.pkg.tar.zst` (Zstandard tarball)            | `.deb` (ar arşivi + tarball)               | `.rpm` (Özel format + cpio)                  | `.apk` (tarball + gzip/zstd)               | `.xbps` (Metadata plist + tarball)             |
| **Paralellik & Hız**                     | **Çok Yüksek** (Rayon ile çok çekirdekli I/O ve SHA kontrolü) | Düşük (Python GIL engeli)                 | Yüksek                                        | Orta                                       | Orta (Python tabanlı üst katman)             | **Çok Yüksek** (Minimal ve hızlı C yapısı) | Yüksek                                         |
| **İşlem Geçmişi & Rollback (Geri Alma)** | **Tam Destek** (Sled üzerinde korumalı rollback/tarihçe)      | Kısmi Destek                              | Log Dosyası (Yerleşik rollback aracı yok)     | Log Dosyası (Yerleşik rollback aracı yok)  | **Tam Destek** (`dnf history undo/rollback`) | Yok (Sadece log)                           | Yok (Sadece log)                               |
| **Delta Paket Desteği**                  | **Var** (Algılama, tamamlama ve diskten yamama)               | Var                                       | Kaldırıldı (Güvenlik/bakım zorluğu nedeniyle) | Dolaylı/Harici (`debdeltas`)               | Var (`deltarpm` / drpm)                      | Yok                                        | Yok                                            |
| **İzole/Sandbox İnşa Altyapısı**         | **Yerleşik** (Chroot ve Linux Namespaces)                     | Yok (Doğrudan ana sistemde inşa)          | Harici Araçla (`extra-x86_64-build`)          | Harici Araçla (`sbuild` / `pbuilder`)      | Harici Araçla (`mock`)                       | Var (`abuild` + bubblewrap)                | **Yerleşik** (`xbps-src` namespaces/chroot)    |
| **İmza Doğrulama / Güvenlik**            | GPG (`gpgv` aracılığıyla)                                     | GPG (Harici/Opsiyonel)                    | PGP/GPG (Yerleşik)                            | GPG (Apt-key/Trusted.gpg)                  | GPG (Yerleşik)                               | RSA Anahtarları (Hızlı imzalama)           | RSA Anahtarları                                |
| **Çapraz Derleme Desteği**               | **Var** (AArch64/ARM64 için build.rs üzerinde yerleşik)       | Yok                                       | Harici araçlar / El ile yapılandırma          | **Çarpan Desteği** (Multiarch altyapısı)   | Harici araçlar / El ile yapılandırma         | **Yerleşik** (cross-compilation destekli)  | **Yerleşik** (`xbps-src` cross derleme motoru) |
| **Servis & Yapılandırma Entegrasyonu**   | **COMAR** (D-Bus ve Subprocess tetikleyicileri)               | COMAR                                     | libalpm hooks                                 | dpkg triggers + maintainer scripts         | RPM triggers + scriptlets                    | APK triggers / scripts                     | XBPS triggers / scripts                        |
| **Yabancı Paket Formatı Desteği**        | **Var** (Doğrudan `.deb` kurulumu ve kaldırılması)            | Yok                                       | Yok (Harici dönüştürücü gerekir)              | Yok (Harici `alien` aracıyla RPM)          | Yok (Harici `alien` aracıyla DEB)            | Yok                                        | Yok                                            |
| **Yapılandırma Dosyası Koruma**          | **Var** (`.config-backup` mekanizması)                        | Var                                       | Var (`.pacnew` / `.pacsave`)                  | Var (conffiles istemleri)                  | Var (`.rpmnew` / `.rpmsave`)                 | Var                                        | Var                                            |

---

## Öne Çıkan Karşılaştırma Detayları

### 1. Performans ve Güvenlik (Rust Faktörü)

Orijinal Python tabanlı Luppo, yorumlanan bir dil (Python 2) kullanması ve dosya sistemi işlemleriyle bütünlük kontrollerinde tek iş parçacığı (Single-Thread) sınırı taşıması nedeniyle yavaştı. `luppo` ise Rust diliyle derlendiğinden doğrudan makine koduna dönüşür ve `rayon` kütüphanesini kullanarak tüm dosya bütünlük taramalarını (SHA1/SHA256) ve I/O operasyonlarını sistemdeki tüm işlemci çekirdeklerine dağıtır.

### 2. Modern Tanımlama Formatları (KDL & JSON)

Eski paket yöneticisi yalnızca XML tabanlı `lopec.xml` formatını desteklerken, `luppo` modern insan odaklı **KDL** (`paket.kdl`) ve makine odaklı **JSON** yapılarını destekler. Bu durum paket yazımını ciddi ölçüde basitleştirir.

### 3. Yerleşik Sandbox ve Chroot

Geleneksel paket yöneticilerinin birçoğu (pacman, apt, dnf) paketi temiz bir ortamda derlemek için harici araçlara (mock, sbuild vb.) ihtiyaç duyarken, `luppo` paket inşa sürecini doğrudan Linux Namespaces ve Chroot kullanarak tamamen izole bir sandbox içinde gerçekleştirir.

### 4. Yabancı Paket Desteği

`luppo`, Linux paket yöneticileri arasında nadir görülen bir özellik olan **Debian (.deb) paketlerini yerel olarak tanıma, kurma ve kaldırma** yeteneğine sahiptir. Bu yetenek, sistemde yerel Luppo paketi bulunmayan üçüncü taraf yazılımların kolayca kurulabilmesini sağlar.

### 5. COMAR ile Konfigürasyon Yönetimi

Debian ve RedHat türevleri paket kurulumu sonrası konfigürasyonları ve servis tetiklemelerini kabuk betikleriyle yaparken, `luppo` bu işlemleri D-Bus arayüzü ve subprocess entegrasyonuna sahip **COMAR (Configuration Manager)** altyapısı üzerinden gerçekleştirir. Bu sayede sistem yapılandırmaları daha modüler ve güvenli şekilde yürütülür.
