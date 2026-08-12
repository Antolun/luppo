# PiSi Kullanıcı Kılavuzu

PiSi, LupuS ve Pisi Linux dağıtımlarının paket yöneticisi PiSi'nin (Packages Installed Successfully as Intended) Rust ile yeniden yazılmış, yüksek performanslı ve güvenli sürümüdür.

---

## 1. Kurulum

### Derleme ve Kurma

```bash
cd pisi
make build                    # veya: cargo build --release
sudo make install             # /usr/bin/pisi, lspisi, unpisi kurulur
```

Kurulum sonrası yapılandırma dosyaları `/etc/pisi/` dizinine yerleşir:
- `/etc/pisi/pisi.conf` — Ana yapılandırma
- `/etc/pisi/mirrors.conf` — Kaynak aynaları

### Gerekli Dizinler

```bash
sudo mkdir -p /var/lib/pisi/db /var/cache/pisi /var/pisi /run/lock/subsys
```

---

## 2. Temel Kullanım

```
pisi [SEÇENEKLER] <KOMUT> [PARAMETRELER]
```

### İlk Kurulum

```bash
# Depo ekle
sudo pisi add-repo core https://repo.pisilinux.org/core/pisi-index.xml

# Depo indeksini güncelle
sudo pisi update-repo
```

### Paket Yönetimi

| Komut | Kısayol | Açıklama |
|-------|---------|----------|
| `pisi install <paket>` | `it` | Paket kur (`.pisi` veya `.deb`) |
| `pisi remove <paket>` | `rm` | Paket kaldır |
| `pisi upgrade` | `up` | Tüm sistemi güncelle |
| `pisi upgrade <paket>` | `up` | Tek paket güncelle |
| `pisi emerge <paket>` | `em` | Kaynaktan derleyip kur |
| `pisi emerge-up` | `emup` | Tüm paketleri kaynaktan yeniden derle |

### Sorgulama ve Listeleme

| Komut | Kısayol | Açıklama |
|-------|---------|----------|
| `pisi search <kelime>` | `sr` | Paket ara |
| `pisi info <paket>` | — | Paket bilgisi göster |
| `pisi list-installed` | `li` | Kurulu paketleri listele |
| `pisi list-available` | `la` | Depodaki paketleri listele |
| `pisi list-upgrades` | `lu` | Güncellenebilir paketleri listele |
| `pisi list-repo` | `lr` | Depoları listele (aktif yeşil, pasif kırmızı) |
| `pisi list-files <paket>` | `lf` | Pakete ait dosyaları listele |
| `pisi search-file <yol>` | `sf` | Dosyanın hangi pakete ait olduğunu bul |
| `pisi blame <paket>` | `bl` | Paket sahibi ve sürüm bilgisi |
| `pisi history` | `hs` | İşlem geçmişini göster |
| `pisi list-orphaned` | `lo` | Sahipsiz paketleri listele |

### Depo Yönetimi

| Komut | Kısayol | Açıklama |
|-------|---------|----------|
| `pisi add-repo <ad> <url>` | `ar` | Depo ekle |
| `pisi remove-repo <ad>` | `rr` | Depo kaldır |
| `pisi enable-repo <ad>` | `er` | Depoyu etkinleştir |
| `pisi disable-repo <ad>` | `dr` | Depoyu devre dışı bırak |
| `pisi update-repo` | `ur` | Depo indeksini güncelle |
| `pisi fetch <paket>` | `fc` | Paketi indir (kurma) |

### Sistem Bakımı

| Komut | Kısayol | Açıklama |
|-------|---------|----------|
| `pisi check-install` | `ci` | Kurulu paketlerin bütünlüğünü kontrol et |
| `pisi configure-pending` | `cp` | Bekleyen paketleri yapılandır |
| `pisi delete-cache` | `dc` | Önbelleği temizle |
| `pisi rebuild-db` | `rdb` | Veritabanını yeniden inşa et |
| `pisi clean` | — | Kullanılmayan kilitleri temizle |
| `pisi remove-orphaned` | `ro` | Sahipsiz paketleri kaldır |
| `pisi rollback <trace-id>` | `rb` | Sistemi geçmiş bir ana döndür |
| `pisi backup-db` | `bd` | Veritabanını yedekle |
| `pisi restore-db` | `rd` | Veritabanını geri yükle |
| `pisi verify-db` | `vdb` | Veritabanı bütünlüğünü doğrula |

### Paket İnşa Etme

| Komut | Kısayol | Açıklama |
|-------|---------|----------|
| `pisi build <dosya>` | `bi` | Paket inşa et (KDL/XML/JSON) |
| `pisi temp` | `tmp` | Yeni paket şablonu oluştur |
| `pisi index <dizin>` | `ix` | `.pisi` dosyalarının kataloğunu çıkar |
| `pisi delta <eski> <yeni>` | `dt` | Delta paket oluştur |
| `pisi toolchain --start` | `tc` | Chroot ortamı başlat |
| `pisi toolchain --update` | `tc` | Toolchain güncelle |
| `pisi graph <paket>` | — | Bağımlılık grafiği çiz (DOT formatı) |
| `pisi check-repo --circular` | — | Döngüsel bağımlılık denetimi |
| `pisi repo-diff <i1> <i2>` | `rdiff` | Depo indekslerini karşılaştır |
| `pisi check-components <dizin>` | — | Bileşen yapısını denetle |
| `pisi reset-history <dizin>` | — | Geçmiş kayıtlarını sıfırla |

---

## 3. Global Seçenekler

| Seçenek | Açıklama |
|---------|----------|
| `-D, --destdir <DİZİN>` | Sistem kökünü değiştir |
| `-y, --yes-all` | Tüm sorulara evet kabul et |
| `-v, --verbose` | Detaylı çıktı |
| `-d, --debug` | Hata ayıklama çıktısı |
| `-N, --no-color` | Renkli çıktıyı kapat |
| `-L, --bandwidth-limit <KB>` | İndirme hızını sınırla (KB/s) |
| `-j, --jobs <N>` | Paralel derleme iş parçacığı sayısı (örn: 4, j8) |
| `-u, --username <KULLANICI>` | Depo kimlik doğrulama kullanıcı adı |
| `-p, --password <ŞİFRE>` | Depo kimlik doğrulama şifresi |
| `--download-only` | Sadece indir, kurma |
| `--ignore-check` | Bütünlük doğrulamasını atla (SHA1/GPG) |
| `--ignore-dependency` | Bağımlılık çözümlemesini atla |
| `--ignore-safety` | Sistem tabanı korumasını atla (tehlikeli!) |
| `--ignore-comar` | COMAR yapılandırma adımlarını atla |
| `--ignore-file-conflict` | Dosya çakışmalarını yok say |
| `--ignore-package-conflict` | Paket çakışmalarını yok say |
| `--no-sandbox` | Sandbox izolasyonu olmadan inşa et |
| `--install-deps` | İnşa bağımlılıklarını otomatik kur |
| `--log-path <DOSYA>` | Derleme loglarını dosyaya yaz |
| `--opt-level <SEVİYE>` | Optimizasyon seviyesi (2, 3, s) |

---

## 4. JSON Çıktı Desteği

Çoğu sorgu komutu `--json` bayrağı ile makine tarafından okunabilir JSON çıktısı üretebilir:

```bash
pisi li --json
pisi sr <paket> --json
pisi info <paket> --json
pisi lr --json
pisi hs --json
pisi lf <paket> --json
```

---

## 5. Dilde Yerelleştirme (i18n)

pisi Türkçe ve İngilizce olmak üzere iki dil desteği sunar. Dil, `LC_ALL` ortam değişkenine göre otomatik seçilir:

```bash
LC_ALL=tr_TR pisi help          # Türkçe
LC_ALL=en_US pisi help          # İngilizce
LC_ALL=de_DE pisi help          # Bilinmeyen dil → varsayılana düşer
```

---

## 6. Yardımcı Araçlar

### lspisi — `.pisi` arşivi içindeki dosyaları listeleme

```bash
lspisi <paket.pisi>
```

### unpisi — `.pisi` arşivini diske çıkarma

```bash
unpisi <paket.pisi> [hedef_dizin]
```

---

## 7. Paket İnşa Dosyaları (Spec Formatları)

pisi üç farklı paket tanım formatını destekler:

| Format | Dosya | Durum |
|--------|-------|-------|
| **KDL** | `paket.kdl` | **Yeni standart** (önerilen) |
| **XML** | `pspec.xml` | Eski format (kullanımdan kaldırıldı) |
| **JSON** | `pspec.json` | Alternatif makine formatı |

KDL formatındaki bir `paket.kdl` dosyası `[source]`, `[[package]]` ve `[build]` bölümlerinden oluşur. Detaylı ActionsAPI referansı için `PISI_PACKAGER_GUIDE.md` dosyasına bakınız.

### Örnek Kullanımlar

```bash
# Mevcut dizindeki spec dosyasını otomatik tanıyarak inşa et
pisi bi

# Belirli bir KDL dosyasından inşa et
pisi bi /path/to/paket.kdl

# Çapraz derleme (AArch64)
pisi bi --target aarch64

# Sandbox'suz inşa (Docker/CI için)
pisi bi --no-sandbox
```

---

## 8. Yapılandırma Dosyası (`/etc/pisi/pisi.conf`)

Üç ana bölümden oluşur:

- **`build`** — Derleyici bayrakları, iş parçacığı sayısı, sandbox ayarları
- **`directories`** — Önbellek, veritabanı, geçici dizin yolları
- **`general`** — Mimari, dağıtım adı, bant genişliği limiti, önbellek ayarları

Yapılandırma dosyası olmasa bile pisi varsayılan değerlerle çalışır.

---

## 9. Veritabanı ve Kilit Sistemi

- **Veritabanı**: Sled (gömülü key-value) — `/var/lib/pisi/db/`
- **Kilit**: `/run/lock/subsys/pisi.lock` (flock) — eşzamanlı işlemleri engeller
- **Salt-okunur erişim**: Root olmayan kullanıcılar için veritabanı `/tmp/pisi-db-readonly-<uid>/` dizinine kopyalanır
- **İşlem Geçmişi (History)**: Tüm işlemler trace ID ile kaydedilir, rollback sırasında kayıtlar silinmez

---

## 10. Rollback (Geri Alma)

```bash
# Geçmişi listele
pisi history

# Belirli bir işleme geri dön
sudo pisi rollback <trace-id>

# veya kısa yolu:
sudo pisi hs -t <trace-id>
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
Evet, `pisi install <dosya.deb>` ile Debian paketlerini doğrudan kurabilirsiniz.

**S: İnşa bağımlılıklarını otomatik kurabilir miyim?**
Evet, `pisi bi --install-deps` ile derleme öncesi gerekli bağımlılıklar sisteme kurulur.

**S: Hız sınırlaması nasıl yapılır?**
`-L <KB/s>` seçeneği ile indirme hızını sınırlayabilirsiniz.

**S: Çıktı renklerini nasıl kapatırım?**
`-N` veya `--no-color` seçeneğini kullanın.

**S: Derleme loglarını nerede bulurum?**
Varsayılan olarak `/var/log/pisi-build.log` veya `--log-path` ile belirttiğiniz dosyada. Hatalı derlemelerde `pisi-build-error.log` oluşur.
