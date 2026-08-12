# pisi: PiSi Paket Yönetim Sistemi (Rust Sürümü)

PiSi (Packages Installed Successfully as Intended), LupuS ve PisiLinux projeleri tarafından geliştirilen modern bir paket yönetim sistemidir. `pisi`, bu sistemi Rust programlama dili ile yeniden hayata geçirmeyi hedefleyen, yüksek performanslı ve güvenli bir sürümüdür.

## Özellikler ve Karşılaştırma

| Özellik | Python PiSi (Eski) | pisi (Yeni) |
| :--- | :--- | :--- |
| **Dil** | Python 2.7 | **Rust (Modern & Güvenli)** |
| **Hız** | Yavaş (Interpreted) | **Çok Hızlı (Compiled)** |
| **Paralellik** | Sınırlı (GIL) | **Tam Paralel (Rayon)** |
| **CLI Altyapısı** | Custom/Optparse | Modern `Clap` (v4) |
| **Çıktı Formatı** | Sadece Metin | Metin + Standart JSON Desteği |
| **Paket Formatı** | .pisi (ZIP + XML) | .pisi (Geriye dönük tam uyumlu) |
| **Güvenlik** | GPG Harici/Opsiyonel | Yerleşik GPG ve SHA256 Doğrulama |
| **Hata Yönetimi** | Çalışma zamanı istisnaları | Güçlü Tip Sistemi (`Result<T, E>`) |
| **İnşa Sistemi** | Karmaşık Python scriptleri | `pisi-builder` (Modüler ve Hızlı) |
| **Veritabanı** | BerkeleyDB / XML | `sled` (Yüksek performanslı KV store) |

### Neden Rust?
Rust sürümü, özellikle büyük paket depolarında bağımlılık çözümleme süresini milisaniyeler seviyesine indirirken, `rayon` sayesinde dosya sistemi işlemlerini ve bütünlük kontrollerini (SHA1/SHA256) tüm CPU çekirdeklerini kullanarak gerçekleştirir. Python 2'nin artık desteklenmemesi ve sistem güvenliği riskleri, `pisi`'i daha güvenli ve sürdürülebilir bir alternatif yapmaktadır.

Çalışma Bağımlılıkları:

	* Python 3

Core Repoda Bulunan Ancak pisi'de İhtiyaç Kalmayan Paketler:

| Paket | Dizin | Açıklama |
| --- | --- | --- |
| python (2.7) | system/base/python | pisi Python 3 kullanır, Python 2 gerekmez |
| piksemel | system/base/piksemel | Yerini Rust quick-xml aldı |
| python3-piksemel | system/base/python3-piksemel | Aynı sebepten |
| plyvel | system/base/plyvel | Yerini Rust sled aldı |
| urlgrabber | system/base/urlgrabber | Yerini Rust reqwest aldı |
| python-psutil | system/base/python-psutil | Yerini Rust nix/libc aldı |
| python-pyliblzma | system/base/python-pyliblzma | Yerini Rust xz2 aldı |
| pycurl | system/base/pycurl | Yerini reqwest aldı |
| pypolkit | system/base/pypolkit | PolicyKit D-Bus/zbus ile |
| dbus-python | system/base/dbus-python | Yerini Rust zbus aldı |
| pisilinux-python | system/base/pisilinux-python | Python 2 helper modülleri |
| catbox | system/devel/catbox | pisi sandbox'ı Rust'ta |
| python-six | system/base/python-six | Python 2/3 uyumluluk |
| python-setuptools (2) | system/devel/python-setuptools | Python 2 için, gerekmez |
| comar-api | system/base/comar-api | Python API'si, gerekmez |

Hâlâ gerekli olanlar:

| Paket | Dizin | Açıklama | 
| --- | --- | --- |
| python3 | system/base/python3 | pisi pyo3 ile Python 3 actions.py çalıştırır |
| python3-setuptools | system/devel/python-setuptools | Python 3 paket build'leri için |
| comar | system/base/comar | Daemon olarak D-Bus üzerinden hâlâ kullanılıyor |
| leveldb | system/base/leveldb | Başka paketler kullanıyorsa kalmalı |

## YAPILANLAR:

    [+] Çoklu paket kurulumu tamam.
    [+] Dinamik sürüm/release yönetimi tamam.
    [+] Esnek XML parsing tamam.
    [+] Önbellek (Cache) yönetimi tamam.
    [+] Paket açıklamaları sistem diline göre görüntüleniyor
    [+] Konfigürasyon dosyaları için .config-backup mekanizması tamam.
    [+] GPG imza doğrulaması (gpgv) entegre edildi.
    [+] Rayon ile paralel dosya sistemi işlemleri ve bütünlük kontrolü tamam.
    [+] Tüm sorgu komutları için standart JSON çıktı desteği eklendi.
	[+] Yeni Standart KDL Paket İnşa Dosyası (paket.kdl) Desteği (XML Resmi Olarak Deprecated Edilecek).
	[+]	Json inşa dosyası desteği
	[+] Çoklu dil desteği eklendi.
	[+] Debian paketlerin tanıma ve kurma yeteneği eklendi.
	[+] AArch64/Arm64 çapraz derleme (cross-compilation) desteği eklendi.
	[+] COMAR entegrasyonu (D-Bus ve Subprocess desteği) tamam.
	[+] Tam izole Sandbox (Chroot ve Namespaces) desteği eklendi.
	[+] Tek paket güncelleme (up <paket>) yeteneği eklendi.
	[+] `--ignore-dependency` ve `--ignore-safety` (Sistem tabanı korumasını atlama) bayrakları entegre edildi.
    [+] Tam Delta Paket Desteği algılama, tamamlama ve kurma entegre edildi.
    [+] Chroot Toolchain ve Bootstrap Alt Sistemi (tc / toolchain komutu) eklendi.
    [+] Döngüsel Bağımlılık Denetimi (check-repo --circular) eklendi.
    [+] Depo Karşılaştırma (repo-diff) eklendi.
    [+] Bileşen Yapısı ve Dizin Senkronizasyonu (check-components) eklendi.
    [+] Paket Güncelleme Geçmişi Sıfırlama (reset-history) eklendi.
    [+] pisi hs -t / rollback koruma ve izleme günlüğü sistemi eklendi.
    [+] Depo Listesi (lr) ve Arama (sr) renklendirmeleri ve etiketleri eklendi.


## PISI fonksiyonları:

kullanım: pisi [seçenekler] <komut> [parametreler]

<komut> aşağıdakilerden birisi olabilir:

	[+]           add-repo (ar) - Depo ekle
	[+]              blame (bl) - Paket sahibi ve yayım bilgisini göster
	[+]              build (bi) - Yeni bir PiSi paketi inşa et (Çapraz derleme --target dahil)
	[+]      check-install (ci) - Kurulu paketlerin bütünlüğünü kontrol et
	[+]    check-components - Bileşen yapı ve dizin bütünlüğünü denetle
	[+]       reset-history - pspec.xml paket güncelleme geçmişlerini sıfırla
	[+]          check-repo - Depo bütünlüğünü ve döngüsel bağımlılıkları denetle
	[+]           repo-diff - İki depo indeksi arasındaki farkları karşılaştır
	[+]           toolchain - Stable Chroot Toolchain ve Bootstrap yönetimi (tc)
	[+]                   clean - Kullanılmayan kilitleri temizle
	[+]  configure-pending (cp) - Kalan paketleri yapılandır
	[+]       delete-cache (dc) - Önbellek dosyalarını temizle
	[+]              delta (dt) - Delta paketleri yarat
	[+]       disable-repo (dr) - Depoyu devre dışı bırak
	[+]             emerge (em) - Depodan paket ve bağımlılıklarını kur
	[+]         emerge-up (emup) - Depodaki kaynak paketlerden toplu güncelleme yap
	[+]        enable-repo (er) - Depoyu etkinleştir
	[+]              fetch (fc) - Paket(ler)i indir
	[+]                   graph - Paket ilişkilerinin grafiğini çıkar
	[+]                help (?) - Verilen komutlar hakkında yardım görüntüler
	[+]            history (hs) - PiSi işlemlerinin günlüğü (Renkli ve korumalı rollback desteğiyle)
	[+]              index (ix) - Verilen dizindeki PiSi dosyalarının kataloğunu çıkar
	[+]                    info - Paket bilgisini göster
	[+]            install (it) - Paket kur
	[+]     list-available (la) - Depolardaki paketleri listele
	[+]    list-components (lc) - Bileşenleri listele
	[+] 	    list-files (lf) - Pakete ait dosyaları listele
	[+]     list-installed (li) - Tüm kurulu paketlerin listesini bas
	[+]        list-newest (ln) - Depolardaki en yeni paketleri listele
	[+]      list-orphaned (lo) - Sahipsiz (orphaned) paketleri listele
	[+]       list-pending (lp) - Yapılandırma bekleyen paketleri listele
	[+]          list-repo (lr) - Depoları listele (Aktif yeşil, pasif kırmızı gösterimle)
	[+]       list-sources (ls) - Müsait kaynakları listele
	[+]      list-upgrades (lu) - Güncellenebilir paketleri listele
	[+]        rebuild-db (rdb) - Veritabanlarını Yeniden İnşa Et
	[+]             remove (rm) - PiSi paketlerini kaldır
	[+]    remove-orphaned (ro) - Sahipsiz (orphaned) paketleri kaldır
	[+]        remove-repo (rr) - Depoları kaldır
	[+]           rollback (rb) - Sistemi belirli bir Trace ID'ye geri döndür
	[+]             search (sr) - Paket ara (Prefix depo isimleriyle birlikte)
	[+]        search-file (sf) - Dosya ara
	[+]        update-repo (ur) - Depo veritabanlarını güncelle
	[+]            upgrade (up) - Sistemi ve paketleri güncelle
	[+]             version - Sürüm bilgilerini detaylı göster

Belirli bir komut hakkında yardım almak için "pisi help <komut>" kullanınız.

Seçenekler:
	[+]	--version                    : programın sürüm numarasını göster ve çık
	[+]	-h [--help]                  : bu yardım iletisini göster ve çık

	genel seçenekler:
	[+]	-D [--destdir] arg          : PiSi komutları için sistem kökünü değiştir.
	[+]	-y [--yes-all]              : Bütün evet/hayır sorularında cevabı evet kabul et.
	[+]	-u [--username] arg         : Depo kimlik doğrulaması için kullanıcı adı.
	[+]	-p [--password] arg         : Depo kimlik doğrulaması için şifre.
	[+]	-L [--bandwidth-limit] arg  : Bant genişliği kullanımını belirtilen kilobaytın altında tut.
	[+]	-v [--verbose]              : Detaylı çıktı
	[+]	-d [--debug]                : Hata ayıklama bilgisini göster.
	[+]	-N [--no-color]             : PiSi çıktılarında renk kullanılmasını engeller.
	[+]	--ignore-dependency         : Bağımlılık çözümlemesini ve çakışma denetimini atlar.
	[+]	--ignore-safety             : Kritik sistem bileşenlerinin (system.base) silinmesini engelleyen korumayı atlar.
	[+]	--no-sandbox                : Paket inşası sırasında sandbox izolasyonunu devre dışı bırakır.
	[+]	--install-deps              : Paket inşası öncesi gerekli inşa bağımlılıklarını sisteme kurar.

## PROJE YAPISI

`pisi` Rust workspace yapısında 4 ana crate'den oluşur:

| Crate | Yol | Açıklama |
|-------|-----|----------|
| **pisi-spec** | `pisi-spec/` | Paket şemaları ve model tipleri (XML/KDL parsing, `Package`, `PackageActions`, `BuildDependencies`, `BuildDepsVisitor`) |
| **pisi-core** | `pisi-core/` | Çekirdek mantık: `Resolver`, `Installer`, `Package`, `InstalledPackage`, `Repo`, `Query`, `History`, `Lock`, `Delta`, GPG doğrulama |
| **pisi-builder** | `pisi-builder/` | Paket inşa motoru: `BuildOptions`, `internal_build`, sandbox/host kurulum, actionsapi |
| **pisi** (binary) | `pisi/` | CLI entry point: `Commands`, arg parsing (Clap), Python API bridge, komut dispatch |

### pisi-builder / actionsapi Modül Yapısı

`pisi-builder/src/actionsapi/` altındaki modüler yapı (eski 1732 satırlık `actionsapi.rs` yerine):

| Modül | Fonksiyonlar |
|-------|-------------|
| `core.rs` | `run_command`, `cd`, `install`, `export_flags`, `symlink`, `set_perms`, `move_path`, `remove_path`, `check_required_tools`, `setup_comar_env` |
| `get.rs` | Tüm getter'lar: `make_jobs()`, `cflags()`, `arch()`, `host()`, `cc()`, `src_name()`, `src_version()`, `install_dir()`, `pkg_dir()`, `doc_dir()`, `man_dir()`, `sbin_dir()` vb. |
| `archive.rs` | `verify_archive`, `unpack_archive`, `do_patch` |
| `shell.rs` | `run_shell`, `run_shell_command` |
| `filesystem.rs` | `dodir`, `dosym`, `domove`, `insinto`, `doexe`, `dolib_a`, `dolib_so`, `remove_dir`, `dopixmaps` |
| `install_tools.rs` | `dobin`, `dosbin`, `dolib`, `doman`, `doinfo`, `dodoc`, `install_headers`, `dosed` |
| `buildtools.rs` | Autotools, CMake, Meson, Ninja, Qt5/6, KDE, Python, SCons, Perl, Kernel, Cargo, Ruby + `remove_packlist`, `remove_podfiles` |
| `utils.rs` | `merge_usr_dirs`, `strip`, `strip_dir`, `gnuconfig_update`, `fix_pkgconfig`, `fix_info_dir`, `dohtml`, `domo`, `newdoc`, `newman` |
| `mod.rs` | Tüm modüllerin re-export'ı + `i18n!` makrosu |

### Önemli Tasarım Kararları

- **Build Dep Yönetimi**: Python `pisi bi` davranışı birebir taklit edildi — unsatisfied build deps listesi → onay → transitif çözüm → tam plan → onay → kurulum (sandbox/host)
- **`pre_remove` Script Desteği**: `PackageActions.pre_remove` → `InstalledPackage.pre_remove` → `installer.run_pre_remove()` → remove akışında COMAR PreRemove'dan önce çağrılır
- **Rust 2024 Uyumluluğu**: Tüm `std::env::set_var` / `remove_var` çağrıları `unsafe { }` bloklarıyla sarıldı (60+ lokasyon)
- **GPG**: `sequoia-openpgp` API uyumsuzluğu nedeniyle pure-Rust ertelendi; `gpgv` shell-out korunuyor
- **Delta Paket**: Hesaplama mantığı var, `.delta.pisi` dosya yazma ve installer uygulaması eksik (ileride tamamlanacak)

### Derleme & Test

```bash
cargo build          # Tüm workspace
cargo test           # 11 test (pisi-spec 4, pisi-builder 7)
cargo build --release
```

