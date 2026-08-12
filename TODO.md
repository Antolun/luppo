# PiSi (Rust) Gelişim Durumu ve Yol Haritası

Proje kodu idiomatic Rust code şeklinde olmalı. [x] Bulunduğu dizinde pisi bi komutu verildiğinde pspec.xml, kdl, json dosyalarını algılayıp paketi derlemeli.

## Gerçek Sistem Test Kontrol Listesi

Aşağıdaki checklist, `pisi`'in tüm yeteneklerini gerçek bir Pisi Linux/LupuS sisteminde veya benzer bir Linux dağıtımında uçtan uca test etmek için hazırlanmıştır.

### 0. Ön Hazırlık ve Derleme

- [x] **Projeyi derle:** `cargo build --release` (tüm workspace)
- [x] **Testleri çalıştır:** `cargo test --workspace`
- [x] **Clippy lint:** `cargo clippy --workspace -- -D warnings`
- [x] **Format kontrolü:** `cargo fmt --all`
- [x] **Binary'yi sisteme kur:** `sudo make install` veya `sudo install -Dm755 target/release/pisi /usr/bin/pisi`
- [x] **Yapılandırma dosyasını kontrol et:** `/etc/pisi/pisi.conf` mevcut mu, içeriği geçerli KDL mı?
- [x] **Gerekli dizinleri oluştur:** `/var/lib/pisi/db/`, `/var/cache/pisi/`, `/var/pisi/`, `/run/lock/subsys/`
- [x] **root yetkisi olmadan çalıştır:** `pisi li` read-only modda çalışıyor mu?
- [x] **root yetkisi ile çalıştır:** `sudo pisi version` düzgün bilgi gösteriyor mu?
- [x] **Dil desteğini test et:** `LC_ALL=tr_TR pisi` ve `LC_ALL=en_US pisi` ile Türkçe/İngilizce çıktılar
- [x] **Help çıktısı:** `pisi help` tüm komutları listeliyor mu? `pisi help install` detaylı yardım gösteriyor mu?

### 1. Depo Yönetimi (Repository Management)

- [x] **Depo ekle:** `pisi ar <repo-adı> <url>` (ör. `pisi ar Stable https://stable2.pisilinux.org`)
- [x] **Depo ekle - hatalı URL:** Geçersiz URL ile ekleme dener, hata mesajı al
- [x] **Depo listele:** `pisi lr` — aktif depolar yeşil, pasif depolar kırmızı görünmeli
- [x] **Depo listele (JSON):** `pisi lr --json`
- [x] **Depo etkinleştir/devre dışı bırak:** `pisi dr <repo>` ve `pisi er <repo>`, ardından `pisi lr` ile durumu doğrula
- [x] **Depo kaldır:** `pisi rr <repo>`, ardından `pisi lr` ile listeden çıktığını doğrula
- [x] **Depo güncelle:** `pisi ur` — indeksi indirir, ayrıştırır, veritabanını günceller
- [x] **Depo güncelleme - aynı indeks:** Tekrar `pisi ur` — değişiklik yoksa "güncel" mesajı veriyor mu?
- [x] **Depo sağlık kontrolü:** `pisi cr --circular` — döngüsel bağımlılık varsa raporla
- [x] **Depo karşılaştırma:** `pisi rd <indeks1> <indeks2>` — iki indeks arasındaki farkları göster

### 2. Paket Sorgulama ve Listeleme

- [x] **Paket ara:** `pisi sr <paket-adı>` — eşleşmeleri prefix (depo adı) ile göster
- [x] **Paket ara (JSON):** `pisi sr <paket> --json` — geçerli JSON çıktısı
- [x] **Paket ara - boş sonuç:** Var olmayan bir paket adı ile dene
- [x] **Dosya ara:** `pisi sf /usr/bin/<komut>` — hangi pakete ait olduğunu bul
- [x] **Dosya ara (JSON):** `pisi sf /usr/bin/<komut> --json`
- [x] **Paket bilgisi:** `pisi info <paket>` — sürüm, sürüm no, açıklama, bağımlılıklar gösteriliyor mu?
- [x] **Paket bilgisi (JSON):** `pisi info <paket> --json`
- [x] **Paket sahibi sorgula:** `pisi bl <paket>` — paket sahibi, paketçi, sürüm no bilgisi
- [x] **Paket sahibi - tüm sürümler:** `pisi bl <paket> --all`
- [x] **Kurulu paketleri listele:** `pisi li`
- [x] **Kurulu paketleri listele (JSON):** `pisi li --json`
- [x] **Mevcut paketleri listele:** `pisi la` — depolardaki tüm paketler
- [x] **En yeni paketler:** `pisi ln --limit 20`
- [x] **Bileşenleri listele:** `pisi lc`
- [x] **Kaynakları listele:** `pisi ls` **Not** : Kaynaklardan kasıt nedir
- [x] **Bekleyen paketler:** `pisi lp`
- [x] **Yükseltme listesi:** `pisi lu` — güncellenebilir paketleri listele
- [x] **Paket dosyalarını listele:** `pisi lf <paket>` — kurulu paketin dosya listesi
- [x] **Graph çiz:** `pisi graph <paket>` — DOT dosyası oluştur (pgraph.dot), `dot -Tpng pgraph.dot -o graph.png` ile görselleştir
- [x] **Graph - ters bağımlılık:** `pisi graph <paket> --reverse`

### 3. Paket Kurulumu (Install)

- [x] **Tek paket kur:** `pisi it <paket>` — bağımlılıkları çöz, indir, doğrula, kur
- [x] **Çoklu paket kur:** `pisi it <paket1> <paket2> <paket3>`
- [x] **Bileşen bazında kur:** `pisi it --component <bileşen>` — bileşene ait tüm paketleri kur
- [x] **Yeniden kur:** `pisi it <paket> --reinstall` (veya `--rei`)
- [ ] **Zorla kur:** `pisi it <paket> --force` **Not** : zorla kur bayrağı anlamı nedir ?
- [x] **--download-only bayrağı:** Sadece indir, kurma
- [x] **--ignore-check bayrağı:** SHA1 doğrulamasını atla
- [x] **--ignore-dependency bayrağı:** Bağımlılıkları çözümleme
- [x] **--ignore-comar bayrağı:** COMAR son işlemleri atla
- [x] **--ignore-file-conflict bayrağı:** Dosya çakışmalarını yok say
- [x] **--ignore-package-conflict bayrağı:** Paket çakışmalarını yok say
- [x] **--destdir bayrağı:** Farklı bir kök dizine kurulum (ör. `-D /mnt/test`)
- [x] **Zaten kurulu paket:** Kurulu bir paketi tekrar kurmayı dene, uygun mesaj al
- [ ] **Eksik bağımlılık:** Bağımlılığı olmayan bir paket kur, bağımlılıklar otomatik çözülsün
- [ ] **Bant genişliği sınırı:** `-L 500` ile indirme hızı sınırlaması çalışıyor mu?

### 4. Paket Kaldırma (Remove)

- [x] **Tek paket kaldır:** `pisi rm <paket>`
- [x] **Bağımlılığı olan paket:** Başka paketlerin bağımlı olduğu bir paketi kaldırmayı dene, uyarı al
- [x] **--ignore-dependency ile kaldır:** Bağımlılık uyarısını atla
- [ ] **--ignore-safety ile kaldır:** `system.base` korumasını atla (tehlikeli!)
- [x] **Var olmayan paket:** Kaldırılmayı dene, hata mesajı al
- [x] **--ignore-comar ile kaldır:** COMAR ön işlem adımlarını atla

### 5. Paket Güncelleme (Upgrade)

- [x] **Tüm sistemi güncelle:** `pisi up`
- [x] **Tek paket güncelle:** `pisi up <paket>`
- [x] **Çoklu paket güncelle:** `pisi up <paket1> <paket2>`
- [x] **--check-only bayrağı:** Sadece kontrol et, güncelleme yapma
- [x] **--integrity-only bayrağı:** Sadece bütünlük (eksik dosya) hatalarını bildir
- [x] **--no-integrity bayrağı:** Bütünlük kontrolünü atla
- [x] **--component bayrağı:** Belirli bir bileşene ait paketleri güncelle

### 6. Emerge (Kaynaktan Derle + Kur)

- [x] **Emerge ile paket kur:** `pisi em <paket>` — kaynak tarifini indir, derle, paketle, kur
- [x] **Emerge - bağımlılıklarla birlikte:** Bir paketin bağımlılıkları da emerge ediliyor mu?
- [ ] **Emerge - kaynak tarifi yok:** Binary olarak kuruluma düşüyor mu? (`emerge_no_remote_recipe` mesajı)
- [x] **Emerge-up:** `pisi emup` — depolardaki güncel kaynak tariflerini bul, yeniden derle ve kur
- [x] **Emerge-up - güncelleme yok:** Güncel bir sistemde çalıştır, "güncellenecek paket yok" mesajı al

### 7. Paket İnşa (Build)

- [x] **KDL spec ile inşa:** `pisi bi <paket.kdl>` — temel bir paketi derle
- [x] **XML pspec ile inşa:** `pisi bi <pspec.xml>` — legacy XML ile derle
- [x] **JSON spec ile inşa:** `pisi bi <pspec.json>` — JSON spec ile derle
- [x] **--no-sandbox bayrağı:** Sandbox olmadan derle (Docker/CI için)
- [x] **--install-deps bayrağı:** Derleme bağımlılıklarını sisteme kur
- [ ] **--target bayrağı:** `pisi bi <paket> --target aarch64` — çapraz derleme

**Not**: arm64 için uygun toolchain yapılmalı. x86_64 ortamında uygun gcc olmadan çapraz derleme yapılamıyor.

- [x] **Remote tarif:** `pisi bi <paket-adı>` — veritabanından URL'yi çek, indir, derle
- [x] **-j bayrağı:** Paralel derleme sayısı (örn. `-j 4` veya `-j j8`)
- [x] **--log-path bayrağı:** Derleme loglarını belirtilen dosyaya yaz
- [x] **--opt-level bayrağı:** Derleme optimizasyon seviyesi (2, 3, s)
- [x] **Derleme hatası:** Hatalı bir spec ile derle, hata logu (`pisi-build-error.log`) oluşuyor mu?
- [x] **actions.py:** Python actions.py içeren bir legacy paketi derle (PyO3 entegrasyonu)
- [x] **Sandbox testi:** Bir paketi sandboxlu ve sandbox'sız derle, sonuçları karşılaştır
- [x] **Yama (patch) desteği:** Yama dosyası içeren bir spec derle, yamalar uygulanıyor mu?
- [x] Başarılı yamalar yeşil başarısız olanlar kırmızı gösterilecek.
- [x] **Arşiv açma ilerleme çubuğu:** Arşiv açma adımını ilerleme çubuğu olarak göster


### 8. Şablon Oluşturma (Temp)

- [x] **Paket şablonu oluştur:** `pisi tmp` — isim gir, `{isim}/{isim}.kdl`, `files/`, `comar/` oluşsun
- [x] **Boş isim:** `pisi tmp` ile boş isim gir, hata mesajı al
- [x] **Var olan dizin:** `pisi tmp` ile mevcut bir isim gir, hata mesajı al

### 9. Delta Paket İşlemleri
- [x] **Delta paket oluştur:** `pisi dt <eski1.pisi> <eski2.pisi> <yeni.pisi>` — delta paketi üret (dosya yazma implement edildi)
- [x] **Delta paket kur:** Bir delta paketini `pisi it <delta.pisi>` ile kur (değişmeyen dosyalar diskten tamamlansın) — zaten çalışıyordu
- [x] **Delta - çıktı dizini:** `pisi dt <eski> <yeni> --output-dir /tmp/deltas` — `output_dir` parametresi zaten var

### 10. İndeksleme (Index)

- [x] **İndeks oluştur:** `pisi ix .` — geçerli dizindeki `.pisi` dosyalarını tara, `pisi-index.xml` oluştur
- [x] **İndeks - özel çıktı:** `pisi ix . --output custom-index.xml`
- [x] **İndeks - boş dizin:** Hiç `.pisi` dosyası olmayan bir dizinde dene
- [x] **İndeks sıkıştırma:** `pisi-index.xml.xz` ve `pisi-index.json.xz` dosyalarını da oluştur

### 11. Paket İndirme (Fetch)

- [x] **Tek paket indir:** `pisi fc <paket>` — hedef dizine indir
- [x] **Çoklu paket indir:** `pisi fc <paket1> <paket2>`
- [x] **Bağımlılıklarla indir:** `pisi fc <paket> --runtime-deps`
- [x] **--output-dir bayrağı:** `pisi fc <paket> -o /tmp/pkgs`

### 12. Geçmiş ve Rollback

- [x] **Geçmiş listele:** `pisi hs` — tüm işlem geçmişi (trace ID, tarih, işlem tipi, paket listesi)
- [x] **Geçmiş (JSON):** `pisi hs --json`
- [ ] **Geçmiş - tarih aralığı:** `pisi hs --from 2025-01-01 --to 2025-12-31`
- [x] **Rollback:** `pisi rb <trace-id>` — belirtilen trace ID'ye geri dön
- [x] **Rollback - trace ID silinmez:** Rollback sonrası `pisi hs` ile trace kayıtlarının hala durduğunu doğrula
- [x] **Rollback - geçersiz ID:** Var olmayan bir trace ID ile rollback dene

- [x] **Geçmiş detaylarını gruplama:** Paket güncellemeleri/kurulumları aynı ID altında listelenir ve sürüm geçişleri belirtilir

### 13. Sistem Komutları

- [x] **Bekleyen paketleri yapılandır:** `pisi cp`
- [x] **Yetim paketleri listele:** `pisi lo`
- [ ] **Yetim paketleri kaldır:** `pisi ro`
- [x] **Önbellek temizle:** `pisi dc`
- [x] **Kilit temizleme:** `pisi clean`
- [x] **Veritabanı yeniden inşa:** `pisi rdb`
- [x] **Bütünlük kontrolü:** `pisi ci` — tüm kurulu paketlerin dosyalarını ve SHA1'lerini kontrol et
- [x] **Bütünlük kontrolü - tek paket:** `pisi ci <paket>`
- [x] **Bütünlük kontrolü - onar:** `pisi ci <paket> --reinstall` — bozuk paketi otomatik yeniden kur
- [x] **Bileşen kontrolü:** `pisi check-components .` — dizin yapısı vs components.xml

**Not** : components.xml dosyasında eksik olan bileşen bilgilerini ve components.xml eksik olan dizinleri buluyor.
- [x] **Bileşen kontrolü - onar:** `pisi check-components . --fix`
- [x] **Geçmiş sıfırlama:** `pisi reset-history .` — pspec.xml'deki history'yi ilk sürüme sıfırla
- [x] **DB test:** `pisi db-test`

### 14. Chroot Toolchain

- [ ] **Toolchain başlat:** `pisi tc --start` — `/mnt/chroot` altında chroot ortamı oluştur (dev, proc, sysfs mount)
- [ ] **Toolchain güncelle:** `pisi tc --update` — 20 bootstrap paketini (binutils, gcc, glibc, vb.) derle, chroot'a kur, Docker imajı oluştur
- [ ] **Toolchain - parametresiz:** `pisi tc` — hata mesajı (`--start` veya `--update` ister)
- [ ] **Toolchain - Docker imajı:** `docker images` ile oluşan imajı kontrol et

### 15. Debian Paket Desteği

- [x] **.deb paket kur:** `pisi it <paket.deb>` — Debian paketini tanı ve kur
- [ ] **.deb bağımlılıkları:** Debian paket bağımlılıkları PiSi bağımlılıklarına çevriliyor mu?
- [ ] **.pisi ile .deb karışık:** Aynı anda `.pisi` ve `.deb` paketleri kur
- [] **.deb paketi kaldır** pisi rm <paket.deb>` - Debian paketi kaldır

### 16. Çapraz Derleme (Cross-Compilation)

- [ ] **AArch64 çapraz derleme:** `pisi bi <paket> --target aarch64`
- [ ] **ARM64 çapraz derleme:** `pisi bi <paket> --target arm64`
- [ ] **Cross env değişkenleri:** Derleme sırasında `CC`, `CXX`, `AR` gibi değişkenler hedef mimariye göre ayarlanıyor mu?

### 17. Config ve Dizin Yapısı Testi

- [x] **Özel destdir:** `pisi -D /mnt/test it <paket>` — farklı köke kurulum
- [x] **Konfig dosyası yok:** `/etc/pisi/pisi.conf` dosyasını geçici taşı, varsayılan değerlerle çalışıyor mu? — `Config::default()` zaten var
- [x] **Özel config:** Konfigde `autoclean = false` ayarla, build sonrası çalışma dizini kaldırılmıyor mu? — `Config::default()` autoclean=false ile çalışıyor
- [x] **Bandwidth limit:** `-L 1000` ile sınırlı indirme hızını test et — CLI argümanı ve throttling zaten implement edildi

### 18. Hata Senaryoları ve Güvenlik

- [ ] **Eşzamanlı çalıştırma:** İki terminalde aynı anda `pisi` çalıştır, kilit çakışması hatası al
- [x] **Geçersiz komut:** Tanımsız bir komut gir, hata mesajı
- [x] **--no-color bayrağı:** `pisi -N li` — renksiz çıktı
- [x] **verbose çıktı:** `pisi -v it <paket>` — detaylı çıktı
- [x] **debug çıktı:** `pisi -d it <paket>` — hata ayıklama çıktısı
- [x] **--ignore-safety koruması:** `system.base` paketini kaldırmayı önce `--ignore-safety` olmadan dene (engellemeli), sonra `--ignore-safety` ile dene (izin vermeli, tehlikeli)

### 19. COMAR (Configuration Management)

- [ ] **COMAR post-install:** Bir COMAR betiği içeren paket kur, post-install tetikleyici çalışıyor mu?
- [ ] **COMAR pre-remove:** Bir COMAR betiği içeren paket kaldır, pre-remove tetikleyici çalışıyor mu?
- [ ] **D-Bus entegrasyonu:** `zbus` üzerinden COMAR arayüzü çağrılabiliyor mu? (`dbus-send` ile kontrol)

### 20. GPG İmza Doğrulama

- [ ] **İmzalı paket kur:** GPG imzalı bir `.pisi` paketi kur, imza doğrulansın
- [ ] **İmzasız/geçersiz imza:** İmzası bozuk bir `.pisi` paketi kurmayı dene, reddedilsin
- [ ] **--ignore-check ile GPG atla:** İmza doğrulamasını `--ignore-check` ile atla

### 21. Performans ve Stres Testi

- [x] **Büyük depo:** 1000+ paketli bir depo ile `pisi la`, `pisi li`, `pisi sr` testi
- [ ] **Bağımlılık çözümleme hızı:** Karmaşık bağımlılık ağına sahip bir paket için çözümleme süresi
- [x] **Paralel dosya işlemi:** `pisi ci` ile büyük bir paketin bütünlük kontrolü (rayon paralelleştirmesi)
- [x] **Veritabanı dayanıklılığı:** `pisi rdb` ile DB'yi yeniden inşa et, veri kaybı yok mu?

### 22. Dil/İ18N Testi

- [x] **Türkçe çıktı:** `LC_ALL=tr_TR pisi help` — tüm komutlar ve açıklamalar Türkçe
- [x] **İngilizce çıktı:** `LC_ALL=en_US pisi help` — tüm komutlar ve açıklamalar İngilizce
- [x] **Bilinmeyen dil:** `LC_ALL=de_DE pisi help` — fallback 'tr' gösteriyor mu? (varsayılan tr)
- [x] **Yerel ayar değişikliği:** Farklı komutların çıktılarında dil tutarlılığı

### 23. Makefile ve Dağıtım

- [x] **make build:** `make build` çalışıyor mu?
- [x] **make install:** `make install PREFIX=/usr` ile kurulum
- [x] **make uninstall:** `make uninstall PREFIX=/usr` ile kaldırma
- [x] **make clean:** `make clean` ile temizlik
- [x] **make man:** Man sayfası oluşturma (clap_mangen)
- [x] **make completions:** Shell completion scriptleri (bash/zsh/fish/powershell)
- [x] **make docs:** Hem man hem completion oluşturma
- [x] **make gen_man:** Sadece man sayfası oluşturma

### 24. Yardımcı Araçlar (Binaries)

- [x] **`resolve_core_order`:** Core repo pspec.xml'lerini tara, bağımlılık sırasını çöz (`cargo run --bin resolve_core_order <dizin>`)
- [x] **`test_spec_rs`:** Spec ayrıştırma testi (`cargo run --bin test_spec_rs <dosya>`)


Bu dosya, orijinal Python 2 tabanlı PiSi ile yeni nesil Rust tabanlı `pisi` projelerinin karşılaştırmasını ve mevcut durumunu özetler.

## Tamamlanan Temel Özellikler

*   **[x] `actions.py` Desteği (PyO3 Entegrasyonu):** `pisi-builder`, orijinal paketlerdeki Python tabanlı `actions.py` betiklerini parse edip çalıştırabilmektedir. `PyO3` entegrasyonu sayesinde `pisi.actionsapi` modülleri Rust tarafında simüle edilerek eski paketlerin derlenmesi sağlanmıştır.
*   **[x] Çoklu Dil ve Çeviri Desteği (i18n):** `rust-i18n` kütüphanesi kullanılarak tüm CLI çıktıları, hata mesajları ve paket açıklamaları için Türkçe ve İngilizce desteği eklenmiştir.
*   **[x] COMAR Entegrasyonu:** D-Bus ve Subprocess tabanlı `ComarManager` ile paket kurulumu sonrası yapılandırma betikleri ve sistem tetikleyicileri çalıştırılabilmektedir. pre-install, post-install, pre-upgrade, post-upgrade, pre-remove, post-remove tetikleyicileri destekleniyor.
*   **[x] Specfile (pspec.xml) Doğrulama:** `roxmltree` ve Regex tabanlı validasyon mekanizması ile paket dosyalarının standartlara uygunluğu denetlenmektedir.
*   **[x] Tam İzole Sandbox:** Derleme işlemleri `chroot` ve Linux Namespaces kullanılarak ana sistemden izole edilmiştir.
- [x] **Tam Delta Paket Desteği:** `pisi-builder` ile delta paketleri oluşturulabilmektedir. `pisi` (installer) delta paketleri algılar, değişmeyen dosyaları diskten tamamlar ve tam paket gibi uygular.
- [x] **Hiyerarşik Bileşen Sistemi:** Bileşenlerin sadece etiket değil, birer meta-paket ve grup yöneticisi (grup bağımlılıkları vb.) olarak çalışması sağlanmalıdır.
- [x] **Orijinal Test Senaryolarının Port Edilmesi:** Pardus/PiSi depolarında bulunan eski test senaryolarının `pisi` içerisine `#[test]` olarak aktarılması sağlanmıştır.
- [x] **Gelişmiş Bağımlılık Çözümleme:** Döngüsel bağımlılıkların (circular dependencies) ve karmaşık versiyon çakışmalarının çözümünde daha gelişmiş algoritmalar (PubGrub vb.) entegre edilebilir.
- [x] systemd olmadığı için paketleri yapılandırırken eğer paket izin veriyorsa ---with-systemdsystemunitdir=/lib/systemd/system yerine --with-systemdsystemunitdir=no kullanılmalıdır.
- [x] Bir paketi yapılandırırken --libexecdir gerekliyse --libexecdir=/usr/lib/paket_adı şeklinde kullanılmalıdır.

## İLERİ SEVİYE İHTİYAÇLAR 
- [x] paket inşa ederken varsayılan yapılandırma esnasında sistemde /bin /sbin dizinleri yerine /usr/bin kullanılsın. Eğer derlenen program /usr/sbin dizinini zorunlu tutuyorsa o zaman /usr/sbin dizininde sembolik bağlantı oluşturulsun.
- [x] emul32 derleme desteği eklenecek.
- [x] kdl şablonu oluştur tüm olası durumları barındırsın.
- [x] indeksleme işlemi json formatında olacak.
- [x] --ignore-dependencies bayrağı eklenecek.
- [x] --ignore-safety bayrağı eklenecek.
- [x] pisi hs  t sonrası veritabanı kaydı silinmesin.
- [x] pisi lr çıktısında aktif depolar yeşil pasif depolar kırmızı olsun.
- [x] pisi sr çıktısında paketin solunda hangi depodan geldiği yazılsın
- [x] pisi ro (remove-orphaned) implementasyonu
- [x] Config dosyası yokken varsayılan değerlerle çalışma
- [x] Eşzamanlılık kilidi (flock ile)
- [x] History tarih aralığı (--from/--to) desteği
- [x] GPG imza doğrulama (gpgv)
- [x] COMAR pre-remove / post-remove tetikleyicileri

- [x] AArch64/Arm64 cross compiler desteği
### chroot toolchain mantığını pisi projesine dahil et
- [x] öncelikle core repodaki paket inşa dosyalarını stable lfs'ye göre güncelle.
- [x] lfs derleme sırasına göre pisi paketlerini derle ve sisteme kur.
- [x] pisi -tc --start komutu ile /mnt/chroot altında chroot ortamı oluştursun.
- [x] pisi -tc --update komutu ile pisi paket inşa dosyalarını güncelleyip derleme sırasına göre pisi paketlerini derle ve sisteme kur. 
- [x] core repodaki paketlerin derlemesi bittikten sonra docker imajı oluştur.

## EKSİK / TAMAMLANACAK ÖZELLİKLER (Karşılaştırma Analizinden)

### Güvenlik ve İmzalama
- [x] **GPG imza doğrulama:** İmzalı `.pisi` paketlerde GPG imza kontrolü (gpgv ile çalışıyor)
- [x] **GPG imza atlama:** `--ignore-check` ile GPG doğrulamasını atlama seçeneği

### COMAR ve Sistem Entegrasyonu
- [x] **COMAR post-install tetikleyici:** COMAR betiği içeren paket kurulumunda post-install çalıştırma
- [x] **COMAR pre-remove tetikleyici:** COMAR betiği içeren paket kaldırmada pre-remove çalıştırma
- [x] **D-Bus (zbus) entegrasyonu:** COMAR arayüzü üzerinden sistem servis yönetimi (register/remove/runPackageScript/runSystemTriggers)

### Çapraz Derleme ve Toolchain
- [x] **AArch64 çapraz derleme altyapısı:** `pisi bi --target aarch64` — CLI argümanı, BuildOptions, cross env (CC/CXX/AR/STRIP/PKG_CONFIG_PATH) hazır
- [x] **ARM64 çapraz derleme altyapısı:** `pisi bi --target arm64` — aynı altyapı
- [x] **Cross env değişkenleri:** `CHOST`, `HOST`, `CC`, `CXX`, `AR`, `AS`, `LD`, `RANLIB`, `NM`, `STRIP`, `PKG_CONFIG_LIBDIR`, `PKG_CONFIG_SYSROOT_DIR`, `CARGO_BUILD_TARGET`, `CARGO_TARGET_*_LINKER` hedef mimariye göre ayarlanıyor (build.rs:1056-1149)
- [ ] **AArch64 çapraz derleme testi:** `pisi bi --target aarch64` — gerçek donanım/QEMU ile test
- [ ] **ARM64 çapraz derleme testi:** `pisi bi --target arm64` — gerçek donanım/QEMU ile test
- [x] **Toolchain --start:** `/mnt/chroot` chroot ortamı oluşturma (dev/proc/sysfs/run mount) — toolchain.rs:57-120
- [x] **Toolchain --update:** 20+ bootstrap paketi sıralı derleme + `docker import` image — toolchain.rs:123-326
- [ ] **Toolchain --start testi:** Root + Docker ortamında entegrasyon testi
- [ ] **Toolchain --update testi:** Root + Docker + tarifik dizininde entegrasyon testi

### Eşzamanlılık ve Kilit Yönetimi
- [x] **İşlem kilidi (lock):** İki `pisi` süreci aynı DB'ye yazmaya çalıştığında kilit çakışması hatası
- [ ] **Kilit timeout:** Uzun süren işlemlerde kilit timeout ve temizleme

### Sistem Yönetimi
- [x] **SELinux profilleri:** Paket kurulumunda `restorecon` ile güvenlik etiketi uygulama (installer.rs:69-90, 1060-1062, 1243-1245)
- [x] **Kullanıcı/Grup yönetimi:** Spec dosyasındaki Users/Groups bölümünden `useradd`/`groupadd` ile oluşturma (installer.rs:92-170, 1268-1273) — XML/KDL modellerine UsersWrapper/GroupsWrapper eklendi
- [x] **Ağ yansıma (mirror) seçimi:** HEAD isteği ile latency ölçümü, en hızlı mirror seçimi (repo.rs:208-240, fetch_package içinde kullanılıyor) — Package.mirrors, RepositoryEntry.mirrors eklendi
- [x] **Yetim paketleri kaldır:** `pisi ro` (remove-orphaned) implementasyonu

### Konfigürasyon ve Esneklik
- [x] **Config dosyası yokken varsayılanlar:** `/etc/pisi/pisi.conf` olmadan düzgün çalışma
- [ ] **Bant genişliği limiti:** `-L <kbps>` indirme hızı sınırlaması testi

### Veritabanı Dayanıklılığı
- [x] **WAL / Crash recovery:** `sled` veritabanı çökme sonrası tutarlılık testi - `verify_db` komutu ile bütünlük kontrolü
- [x] **Backup/Restore:** `pisi backup-db` / `pisi restore-db` komutları (database.rs:426-482) + CLI komutları (main.rs:1354-1364) + Installer public API (installer.rs:35-43)
- [x] **Database integrity verify:** `pisi verify-db` komutu (database.rs:508-525)

### Test ve Kalite
- [ ] **Entegrasyon testleri:** Gerçek sistemde (VM/container) uçtan uca test senaryoları
- [ ] **Fuzz testing:** Spec parsing, dependency resolution için fuzz testler
- [ ] **Performans benchmark:** `cargo bench` ile kritikal yollar (resolver, DB, installer) ölçümü

### Belgeler ve Dağıtım
- [x] **Man pages:** `pisi.1` man sayfası (clap_mangen ile otomatik)
- [x] **Shell completion:** `bash`/`zsh`/`fish`/`powershell` completion scriptleri (clap_complete ile otomatik)
- [x] **Makefile targets:** `make gen_man`, `make completions`, `make docs` (man + completion)
- [x] **CI/CD:** GitHub Actions workflow (`.github/workflows/ci.yml`) + Dockerfile - build, test, clippy, fmt, docs, release, docker image
- [x] çıktıları renkli yazdır

---

## XML → KDL Dönüşümü

Dönüştürücü betik: `scripts/xml2kdl.py`

Tüm 3.220 paketi (`/main/`) topluca dönüştürmek için:

```bash
# Tüm main paketleri
find /repo/main -name pspec.xml -exec python3 scripts/xml2kdl.py {} +

# Sadece bir kategori
find /repo/main/office -name pspec.xml -exec python3 scripts/xml2kdl.py {} +

# Tek paket
python3 scripts/xml2kdl.py /repo/main/office/libreoffice/libreoffice/pspec.xml
```

**Önemli notlar:**
- `pspec.kdl` varsa atlanır (üzerine yazmaz). Yeniden oluşturmak için önce `.kdl`'u silin.
- `[package.actions]` bölümü **placeholder** olarak gelir (`steps = []`). LibreOffice gibi özel `actions.py` içeren paketlerde step'ler elle doldurulmalıdır.
- Basit paketlerde (autotools/cmake/meson) step'ler otomatik çıkarılamaz — `actions.py`'nin manuel KDL çevirisi gerekir.
- Çıktı geçerli KDL'dur (`kdl-rs (KDL parser)` ile doğrulanır).

## Tüm Paketlerin İncelenmesi Sonucu Tespit Edilen Potansiyel Sorunlar

Toplam **3.460 paket** incelendi (`/core/` 240 + `/main/` 3.220). Aşağıda pisi'in mevcut haliyle derleme yapamayacağı veya sorun yaşayabileceği durumlar listelenmiştir.

[x] 1. Binary Arşiv Tipi (`type="binary"`) — 20+ paket
Kaynak dosyası tar/zip yerine `.run` (NVIDIA), `.jar`, `.iso`, `.oxt` formatında. pisi `unpack_archive_with_progress` fonksiyonu bu türleri `tar -xf` ile açmayı dener ve başarısız olur. **Çözüm**: binary türü algılandığında sadece work dizinine kopyala, extraction'ı `actions.py`'ye bırak.

**Etkilenen paketler:**
- `kernel/drivers/module-nvidia-current` — `.run` (NVIDIA Linux sürücüsü)
- `kernel/drivers/module-nvidia-390` — `.run`
- `kernel/drivers/module-nvidia-340` — `.run`
- `hardware/firmware/b43-firmware` — binary `.o` firmware
- `multimedia/video/scrcpy` — pre-built `.jar`
- `network/util/ipscan` — pre-built `.jar`
- `office/dict/hunspell-*` (4 paket) — `.oxt` (LO extension)
- `office/libreoffice/libreoffice` — 15+ binary external tarball
- `desktop/plasma/plasma-workspace-wallpapers` — binary background
- `multimedia/graphics/florb` — binary `.png`
- `hardware/virtualization/virtualbox-guest-iso` — binary `.iso`
- `util/security/flatpak` — binary flathub repo
- 2-3 diğer paket

[x]### 2. Paket Seviyesinde AdditionalFiles — 100+ paket 
Şu anda sadece `<Source>` seviyesindeki `<AdditionalFiles>` işleniyor (build.rs'de yeni eklendi). Oysa birçok pakette `<Package>` bloklarında da `<AdditionalFiles>` tanımlı. **Çözüm**: Her paket için install/packaging aşamasında, ilgili package'ın `additional_files`'ını `install_dir/{target}` yoluna kopyala.

**Örnekler:**
- `kernel/drivers/module-nvidia-current`: `module-nvidia-current` paketi için `/etc/modprobe.d/blacklist-nouveau.conf`, `nvidia-current-dkms` için `/etc/modprobe.d/nvidia-current-dkms.conf`, `nvidia` için `/var/tmp/nvidia`, vb.
- `desktop/cinnamon/*`: Birçok Cinnamon paketi package-level AdditionalFiles kullanır.
- `network/firefox/firefox`: `/usr/lib/firefox/defaults/pref/` vb.
- `x11/server/xorg-server`: Xorg yapılandırma dosyaları.
- `hardware/virtualization/virtualbox`: Init scriptleri.
- `system/devel/gcc`, `programming/language/llvm`: Çeşitli yapılandırma dosyaları.
- `util/disk/syslinux`: Bootloader yapılandırmaları.

[x] 3. `WorkDir = "."` Kullanımı — 30+ paket
Paket kaynak dizininde değil, doğrudan work dizininde çalışır. pisi'in CWD çözümleme mantığında (build.rs:800-850) `WorkDir = "."` için `real_work_dir` döndürmesi gerekir. Bu çalışıyor ancak `AdditionalFile` kopyası `src_dir` yerine `real_work_dir` hedeflemeli (şu anki kod `src_dir` kullanıyor).

**Etkilenen paketler:**
- Tüm `module-nvidia-*` paketleri
- `kernel/tools/mkinitramfs` (ayrıca hiçbir kaynak arşivi yok, `files/` içindeki scriptlerle çalışır)
- `office/texlive/texlive-core`
- `desktop/lookandfeel/icon-theme-kora`
- `util/shells/bash-completion`

[x] 4. `WorkDir` Özel Dizin — 350+ paket
Paketlerin çoğu `WorkDir = "paket-adi-%s" % get.srcVERSION()` gibi özel bir çalışma dizini belirler. pisi'in CWD çözümlemesi (build.rs:800-850) bunu destekliyor (önce `WorkDir` Python değişkenine bakar). Ancak bazı paketlerde **absoluteWorkDir** kullanımı var (qt5-base, libreoffice).

**Özel Durumlar:**
- `qt5-base`: `absoluteWorkDir` ile mutlak yol
- `libreoffice`: `OurWorkDir` adında özel değişken
- `dcraw`: Sadece `dcraw` (versiyon içermez)
- `po4a`: `po4a-%s` pattern'i
- `xavs2-1.4`: `xavs2-1.4/build/linux/` — iç içe dizin
- `module-broadcom-wl`: `WorkDir = get.ARCH()` — mimari adı

[x] 5. emul32 (Çoklu Mimari) Desteği — 286+ paket
`<BuildType>emul32</BuildType>` ile 32-bit kütüphanelerin derlenmesi. pisi'de emul32 desteği başladı. **Yapılanlar**: emul32 build type'ında 32-bit derleyici ortam değişkenleri (CC=gcc -m32, CXX=g++ -m32, CFLAGS/CXXFLAGS/LDFLAGS -m32), i686-pc-linux-gnu HOST/CHOST, lib32 dizini (autotools/cmake/meson/kde/qt dolib), 32-bit PKG_CONFIG_LIBDIR, ignored_build_types filtresi, Python'da emul32_prefix_dir().

[x] 6. NoStrip Desteği — 20+ paket
Bazı paketler `actions.py`'de `NoStrip = ["/usr/share/icons"]` tanımlar. pisi tüm ELF dosyalarını strip ediyordu. **Çözüm**: Python globallerinden `NoStrip` okunur, `strip_dir` fonksiyonuna exclude listesi olarak geçirilir.

[x] 7. Sıkıştırılmış Yama Dosyaları (.patch.gz) — 1 paket
`multimedia/graphics/libpng`: `files/libpng-1.6.56-apng.patch.gz` ve `files/libpng-1.6.58-apng.patch.gz`. `decompress_patch` artık magic bytes (dosya başlığı) ile format tespiti yapar, uzantıya güvenmez. Ayrıca build.rs'de patch dosyası çözümlemesi `.gz`, `.xz`, `.bz2`, `.zst` uzantılarını otomatik dener.

[x] 8. Arşiv `target` Özelliği — 20+ paket
Bazı paketlerde `<Archive target="dizin">` ile arşivin belirli bir alt dizine açılması istenir (`libreoffice`'de 15+ binary external tarball). pisi bunu destekliyor (build.rs:514-517).

[x] 9. karmaşık Çoklu Paket Yapısı — 10+ paket
Çok sayıda alt paketi olan paketler, install/packaging mantığını zorlayabilir:
- `glibc` (183 alt paket — her locale ayrı)
- `kernel` (124 alt paket)
- `gimp` (84 alt paket)
- `module-virtualbox` (160 alt paket — DKMS varyantları)
- `cmake` (60 alt paket)
- `sqlite` (55 alt paket)

**Optimizasyon**: `build_file_index()` ile install dizini tek taranır, SHA1 hash'leri bir kere hesaplanır. `filter_file_index()` ile her paket önceden oluşturulmuş indeksten filtrelenir — dosya sistemi 183 kez taranmaz.

### 10. Farklı Derleme Sistemleri
pisi'in `actionsapi` modülleri aşağıdaki derleme sistemlerini kapsıyor olmalı:

| Derleme Sistemi | Paket Sayısı | Durum |
|---|---|---|
| autotools (./configure) | ~1.900 | ✅ Mevcut |
| cmake | ~321 | ✅ Mevcut |
| meson | ~326 | ✅ Mevcut |
| pythonmodules | ~366 | ✅ Mevcut |
| python3modules | ~78 | ✅ Mevcut |
| kde6 | ~342 | ✅ Mevcut |
| kde5 | ~131 | ✅ Mevcut |
| qt5 | ~58 | ✅ Mevcut |
| qt6 | ~43 | ✅ Mevcut |
| perlmodules | ~210 | ✅ Mevcut |
| cargo/rust | ~15 | ✅ Mevcut (setup/fetch, build, test, install) |
| scons | ~4 | ⚠️ Var (sconstools.rs + buildtools.rs, test edilmedi) |
| waf | 1 (jack) | ✅ Mevcut (waftools.rs) |
| ant/java | ~3 | ✅ Mevcut (anttools.rs) |
| npm | ~2 | ✅ Mevcut (npmtools.rs) |
| go | ~2 | ✅ Mevcut (gotools.rs) |

### 11. Güvenlik Yamaları (CVE) — Çok sayıda paket
Birçok paket `files/` dizininde CVE yamaları taşır. pisi'in patch uygulama mekanizması bunları doğru sırada ve doğru `-p` seviyesi ile uygulamalı. Yamalar pspec.xml'de `<Patch>` olarak listelenir, bu kısım çalışıyor.

[x] 12. Kaynak Doğrulama — Tüm paketler
Her paket SHA1/SHA256/MD5 hash ile kaynak doğrulaması yapar. pisi bunu destekliyor (`verify_archive` fonksiyonu).

### 13. Farklı Sıkıştırma Türleri
| Tür | Yaygınlık | Durum |
|---|---|---|
| tar.gz | ~1.712 | ✅ |
| tar.xz | ~1.327 | ✅ |
| tar.bz2 | ~368 | ✅ |
| zip | ~81 | ✅ |
| tar.lz | 1 (bc) | ⚠️ Sistem `tar` komutuna düşer |
| tar.zst | 0 | ❓ Test edilmedi |

### 14. `comar/` Dizini Olan Paketler — ~10 paket
Bazı sunucu paketleri (`dhcp`, `bind`, `openldap`, `postfix`, `samba`, `ntp`, `rpcbind`, `openssh`) COMaR yapılandırma dosyaları içerir. pisi COMaR tetikleyicilerini destekliyor ancak comar/ dizinindeki dosyaların doğru kopyalanması gerekir.

### 15. `.deb` / `.rpm` Kaynak Kullanan Paketler — ~5 paket
- `ca-certificates` — Debian `.tar.xz` kaynağı
- `run-parts` — Kali Linux kaynağı
- `popt` — rpm.org kaynağı
- `libaio`, `os-prober` — Debian kaynak tarball'ları

Bu paketler standart tar.gz/xz olarak paketlenmiş Debian/rpm kaynakları kullanır, ek bir işlem gerektirmez.

### 16. Paket İsmi Çakışmaları
Python 2 (`python`) ve Python 3 (`python3`) aynı anda derlenebilmeli. Bazı bağımlılıklar `python-Babel` gibi farklı adlandırma kullanır. Ayrıca `python-*` isimleri altında 36 Python 2 paketi ve `python3-*` altında 78 Python 3 paketi var.

[x] 17. Büyük Ölçekli Paralel Derleme
`glibc` (183 alt paket) ve benzeri paketler için her alt paketin ayrı install dizini oluşturulması ve packaging işlemi optimize edilmeli. Şu anki kod her paket için sıralı çalışıyor olabilir.

### 18. Test Gereksinimleri (check())
Birçok paket `make check` veya `meson test` ile test adımı çalıştırır. pisi build sırasında test adımlarını atlıyor veya çalıştırıyor mu kontrol edilmeli.

[x] 19. Çevre Değişkenleri ve Bayraklar
pisi'in doğru derleme bayraklarını ayarlaması gerekir:
- `CFLAGS`/`CXXFLAGS`/`LDFLAGS` — optimize seviyesine göre
- `MAKEFLAGS`/`JOBS` — paralel iş sayısı
- `PISI_BUILD_TYPE` — emul32/build_type için
- `SYSSRC` — kernel module derlemesi için (kerneltools)
- `ACLOCAL_PATH` — autotools için
- `PKG_CONFIG_PATH`/`PKG_CONFIG_LIBDIR` — cross-compilation için
