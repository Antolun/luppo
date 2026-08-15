# Luppo Kullanıcı Kılavuzu

Luppo, LupuS ve Luppo Linux dağıtımlarının paket yöneticisi Luppo'nin (Packages Installed Successfully as Intended) Rust ile yeniden yazılmış, yüksek performanslı ve güvenli sürümüdür.

---

## 1. Kurulum

### Derleme ve Kurma

```bash
cd luppo
make build                    # veya: cargo build --release
sudo make install             # /usr/bin/luppo, lsluppo, unluppo kurulur
```

Kurulum sonrası yapılandırma dosyaları `/etc/luppo/` dizinine yerleşir:

- `/etc/luppo/luppo.conf` — Ana yapılandırma
- `/etc/luppo/mirrors.conf` — Kaynak aynaları

### Gerekli Dizinler

```bash
sudo mkdir -p /var/lib/luppo/db /var/cache/luppo /var/luppo /run/lock/subsys
```

---

## 2. Temel Kullanım

```
luppo [SEÇENEKLER] <KOMUT> [PARAMETRELER]
```

### İlk Kurulum

```bash
# Depo ekle
sudo luppo add-repo core https://repo.antolun.com/core/luppo-index.xml

# Depo indeksini güncelle
sudo luppo update-repo
```

### Paket Yönetimi

| Komut                   | Kısayol | Açıklama                              |
| ----------------------- | ------- | ------------------------------------- |
| `luppo install <paket>` | `it`    | Paket kur (`.luppo` veya `.deb`)      |
| `luppo remove <paket>`  | `rm`    | Paket kaldır                          |
| `luppo upgrade`         | `up`    | Tüm sistemi güncelle                  |
| `luppo upgrade <paket>` | `up`    | Tek paket güncelle                    |
| `luppo emerge <paket>`  | `em`    | Kaynaktan derleyip kur                |
| `luppo emerge-up`       | `emup`  | Tüm paketleri kaynaktan yeniden derle |

### Sorgulama ve Listeleme

| Komut                      | Kısayol | Açıklama                                      |
| -------------------------- | ------- | --------------------------------------------- |
| `luppo search <kelime>`    | `sr`    | Paket ara                                     |
| `luppo info <paket>`       | —       | Paket bilgisi göster                          |
| `luppo list-installed`     | `li`    | Kurulu paketleri listele                      |
| `luppo list-available`     | `la`    | Depodaki paketleri listele                    |
| `luppo list-upgrades`      | `lu`    | Güncellenebilir paketleri listele             |
| `luppo list-repo`          | `lr`    | Depoları listele (aktif yeşil, pasif kırmızı) |
| `luppo list-files <paket>` | `lf`    | Pakete ait dosyaları listele                  |
| `luppo search-file <yol>`  | `sf`    | Dosyanın hangi pakete ait olduğunu bul        |
| `luppo blame <paket>`      | `bl`    | Paket sahibi ve sürüm bilgisi                 |
| `luppo history`            | `hs`    | İşlem geçmişini göster                        |
| `luppo list-orphaned`      | `lo`    | Sahipsiz paketleri listele                    |

### Depo Yönetimi

| Komut                       | Kısayol | Açıklama                |
| --------------------------- | ------- | ----------------------- |
| `luppo add-repo <ad> <url>` | `ar`    | Depo ekle               |
| `luppo remove-repo <ad>`    | `rr`    | Depo kaldır             |
| `luppo enable-repo <ad>`    | `er`    | Depoyu etkinleştir      |
| `luppo disable-repo <ad>`   | `dr`    | Depoyu devre dışı bırak |
| `luppo update-repo`         | `ur`    | Depo indeksini güncelle |
| `luppo fetch <paket>`       | `fc`    | Paketi indir (kurma)    |

### Sistem Bakımı

| Komut                       | Kısayol | Açıklama                                 |
| --------------------------- | ------- | ---------------------------------------- |
| `luppo check-install`       | `ci`    | Kurulu paketlerin bütünlüğünü kontrol et |
| `luppo configure-pending`   | `cp`    | Bekleyen paketleri yapılandır            |
| `luppo delete-cache`        | `dc`    | Önbelleği temizle                        |
| `luppo rebuild-db`          | `rdb`   | Veritabanını yeniden inşa et             |
| `luppo clean`               | —       | Kullanılmayan kilitleri temizle          |
| `luppo remove-orphaned`     | `ro`    | Sahipsiz paketleri kaldır                |
| `luppo rollback <trace-id>` | `rb`    | Sistemi geçmiş bir ana döndür            |
| `luppo backup-db`           | `bd`    | Veritabanını yedekle                     |
| `luppo restore-db`          | `rd`    | Veritabanını geri yükle                  |
| `luppo verify-db`           | `vdb`   | Veritabanı bütünlüğünü doğrula           |

### Paket İnşa Etme

| Komut                            | Kısayol | Açıklama                               |
| -------------------------------- | ------- | -------------------------------------- |
| `luppo build <dosya>`            | `bi`    | Paket inşa et (KDL/XML/JSON)           |
| `luppo temp`                     | `tmp`   | Yeni paket şablonu oluştur             |
| `luppo index <dizin>`            | `ix`    | `.luppo` dosyalarının kataloğunu çıkar |
| `luppo delta <eski> <yeni>`      | `dt`    | Delta paket oluştur                    |
| `luppo toolchain --start`        | `tc`    | Chroot ortamı başlat                   |
| `luppo toolchain --update`       | `tc`    | Toolchain güncelle                     |
| `luppo graph <paket>`            | —       | Bağımlılık grafiği çiz (DOT formatı)   |
| `luppo check-repo --circular`    | —       | Döngüsel bağımlılık denetimi           |
| `luppo repo-diff <i1> <i2>`      | `rdiff` | Depo indekslerini karşılaştır          |
| `luppo check-components <dizin>` | —       | Bileşen yapısını denetle               |
| `luppo reset-history <dizin>`    | —       | Geçmiş kayıtlarını sıfırla             |

---

## 3. Global Seçenekler

| Seçenek                      | Açıklama                                         |
| ---------------------------- | ------------------------------------------------ |
| `-D, --destdir <DİZİN>`      | Sistem kökünü değiştir                           |
| `-y, --yes-all`              | Tüm sorulara evet kabul et                       |
| `-v, --verbose`              | Detaylı çıktı                                    |
| `-d, --debug`                | Hata ayıklama çıktısı                            |
| `-N, --no-color`             | Renkli çıktıyı kapat                             |
| `-L, --bandwidth-limit <KB>` | İndirme hızını sınırla (KB/s)                    |
| `-j, --jobs <N>`             | Paralel derleme iş parçacığı sayısı (örn: 4, j8) |
| `-u, --username <KULLANICI>` | Depo kimlik doğrulama kullanıcı adı              |
| `-p, --password <ŞİFRE>`     | Depo kimlik doğrulama şifresi                    |
| `--download-only`            | Sadece indir, kurma                              |
| `--ignore-check`             | Bütünlük doğrulamasını atla (SHA1/GPG)           |
| `--ignore-dependency`        | Bağımlılık çözümlemesini atla                    |
| `--ignore-safety`            | Sistem tabanı korumasını atla (tehlikeli!)       |
| `--ignore-comar`             | COMAR yapılandırma adımlarını atla               |
| `--ignore-file-conflict`     | Dosya çakışmalarını yok say                      |
| `--ignore-package-conflict`  | Paket çakışmalarını yok say                      |
| `--no-sandbox`               | Sandbox izolasyonu olmadan inşa et               |
| `--install-deps`             | İnşa bağımlılıklarını otomatik kur               |
| `--log-path <DOSYA>`         | Derleme loglarını dosyaya yaz                    |
| `--opt-level <SEVİYE>`       | Optimizasyon seviyesi (2, 3, s)                  |

---

## 4. JSON Çıktı Desteği

Çoğu sorgu komutu `--json` bayrağı ile makine tarafından okunabilir JSON çıktısı üretebilir:

```bash
luppo li --json
luppo sr <paket> --json
luppo info <paket> --json
luppo lr --json
luppo hs --json
luppo lf <paket> --json
```

---

## 5. Dilde Yerelleştirme (i18n)

luppo Türkçe ve İngilizce olmak üzere iki dil desteği sunar. Dil, `LC_ALL` ortam değişkenine göre otomatik seçilir:

```bash
LC_ALL=tr_TR luppo help          # Türkçe
LC_ALL=en_US luppo help          # İngilizce
LC_ALL=de_DE luppo help          # Bilinmeyen dil → varsayılana düşer
```

---

## 6. Yardımcı Araçlar

### lsluppo — `.luppo` arşivi içindeki dosyaları listeleme

```bash
lsluppo <paket.luppo>
```

### unluppo — `.luppo` arşivini diske çıkarma

```bash
unluppo <paket.luppo> [hedef_dizin]
```

---

## 7. Paket İnşa Dosyaları (Spec Formatları)

luppo üç farklı paket tanım formatını destekler:

| Format   | Dosya        | Durum                                |
| -------- | ------------ | ------------------------------------ |
| **KDL**  | `paket.kdl`  | **Yeni standart** (önerilen)         |
| **XML**  | `lopec.xml`  | Eski format (kullanımdan kaldırıldı) |
| **JSON** | `lopec.json` | Alternatif makine formatı            |

KDL formatındaki bir `paket.kdl` dosyası `[source]`, `[[package]]` ve `[build]` bölümlerinden oluşur. Detaylı ActionsAPI referansı için `LUPPO_PACKAGER_GUIDE.md` dosyasına bakınız.

### Örnek Kullanımlar

```bash
# Mevcut dizindeki spec dosyasını otomatik tanıyarak inşa et
luppo bi

# Belirli bir KDL dosyasından inşa et
luppo bi /path/to/paket.kdl

# Çapraz derleme (AArch64)
luppo bi --target aarch64

# Sandbox'suz inşa (Docker/CI için)
luppo bi --no-sandbox
```

---

## 8. Yapılandırma Dosyası (`/etc/luppo/luppo.conf`)

Üç ana bölümden oluşur:

- **`build`** — Derleyici bayrakları, iş parçacığı sayısı, sandbox ayarları
- **`directories`** — Önbellek, veritabanı, geçici dizin yolları
- **`general`** — Mimari, dağıtım adı, bant genişliği limiti, önbellek ayarları

Yapılandırma dosyası olmasa bile luppo varsayılan değerlerle çalışır.

---

## 9. Veritabanı ve Kilit Sistemi

- **Veritabanı**: Sled (gömülü key-value) — `/var/lib/luppo/db/`
- **Kilit**: `/run/lock/subsys/luppo.lock` (flock) — eşzamanlı işlemleri engeller
- **Salt-okunur erişim**: Root olmayan kullanıcılar için veritabanı `/tmp/luppo-db-readonly-<uid>/` dizinine kopyalanır
- **İşlem Geçmişi (History)**: Tüm işlemler trace ID ile kaydedilir, rollback sırasında kayıtlar silinmez

---

## 10. Rollback (Geri Alma)

```bash
# Geçmişi listele
luppo history

# Belirli bir işleme geri dön
sudo luppo rollback <trace-id>

# veya kısa yolu:
sudo luppo hs -t <trace-id>
```

Rollback işlemi, hedef durumla mevcut durum arasındaki farkı hesaplar ve yalnızca gerekli paketleri kurar/kaldırır. Geçmiş kayıtları silinmez.

---

## 11. Güvenlik

- Derlemeler varsayılan olarak **sandbox** (chroot + Linux Namespaces) içinde çalışır
- Paket bütünlüğü SHA1/SHA256 ile doğrulanır
- GPG imza doğrulaması (`gpgv`) desteklenir
- `system.base` bileşenleri yanlışlıkla silinmeye karşı korunur (`--ignore-safety` ile atlanabilir)
- İşlem kilidi eşzamanlı çalışmayı engeller

---

## 12. SSS / İpuçları

**S: Root olmadan kullanabilir miyim?**
Evet, sorgulama komutları (`li`, `sr`, `info`, `lr`, `lu`, vb.) root gerektirmez. Kurulum/kaldırma/güncelleme gibi değişiklik yapan komutlar root yetkisi ister.

**S: `.deb` paketi kurabilir miyim?**
Evet, `luppo install <dosya.deb>` ile Debian paketlerini doğrudan kurabilirsiniz.

**S: İnşa bağımlılıklarını otomatik kurabilir miyim?**
Evet, `luppo bi --install-deps` ile derleme öncesi gerekli bağımlılıklar sisteme kurulur.

**S: Hız sınırlaması nasıl yapılır?**
`-L <KB/s>` seçeneği ile indirme hızını sınırlayabilirsiniz.

**S: Çıktı renklerini nasıl kapatırım?**
`-N` veya `--no-color` seçeneğini kullanın.

**S: Derleme loglarını nerede bulurum?**
Varsayılan olarak `/var/log/luppo-build.log` veya `--log-path` ile belirttiğiniz dosyada. Hatalı derlemelerde `luppo-build-error.log` oluşur.
