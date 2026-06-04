<div align="center">
<pre>
  ███████╗██████╗  ██████╗  █████╗ ██╗  ██╗
  ██╔════╝██╔══██╗██╔═══██╗██╔══██╗██║ ██╔╝
  ███████╗██████╔╝██║   ██║███████║█████╔╝ 
  ╚════██║██╔═══╝ ██║   ██║██╔══██║██╔═██╗ 
  ███████║██║     ╚██████╔╝██║  ██║██║  ██╗
  ╚══════╝╚═╝      ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝
</pre>

# Spoak CLI (v0.3.0)

**Terminalde Yaşayan Spoak Deneyimi.**  
Minecraft sunucu yöneticileri ve oyuncuları için tasarlanmış renkli, etkileşimli ve çarpıcı bir CLI (Komut Satırı Arayüzü). "Bento Box" mimarisiyle modern Web3 estetiğini terminale getiriyor.

[![GitHub](https://img.shields.io/badge/GitHub-spoakk%2Fbackend-181717?logo=github)](https://github.com/spoakk/backend) [![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange?logo=rust)](https://rust-lang.org)

</div>

---

## ✨ Neler Yeni? (v0.3.0)
- **🚀 Oto-Güncelleyici:** Artık CLI, her açılışında arka planda güncellemeleri saniyeler içinde sessizce kontrol edip doğrudan `update` mekanizmasına yönlendiriyor.
- **📦 Yeni JAR'lar:** Paper ve Leaf'in yanı sıra artık **Purpur** ve **Folia** sürümlerini de saniyeler içinde çekebiliyorsunuz!
- **🎨 Bento Box 2.0:** Tüm arayüz, zarif ASCII sanatlarıyla ve geçişli degrade (gradient) renk paletiyle baştan aşağıya yenilendi.
- **⚡ Akıllı Yükleyici:** Yükleme scripti (install.ps1) eski hatalarından arındırıldı ve tamamen standart karakterlerle kusursuz hale getirildi.

---

## ✨ Özellikler

- **Göz Alıcı Çıktılar (Terminal Bento):** Tüm çıktılar sade loglar yerine interaktif, unicode pencerelerde RGB degrade renk geçişleriyle sunulur.
- **Otomatik Backend:** CLI ilk başlatıldığında arka planda otomatik olarak `spoak-backend`'i kurar, günceller ve gizlice çalıştırır.
- **Kullanımı Kolay İnteraktif Mod:** Sadece `spoak` yazarak CLI tabanlı modern shell ortamına ( `spoak ❯` ) geçiş yapın.

---

## 🛠️ Kurulum

### Windows (PowerShell)
Tek satırda kurulum için:
```powershell
iwr -useb https://raw.githubusercontent.com/Spoakk/cli/main/install.ps1 | iex
```

### Manuel Yükleme
Alternatif olarak, doğrudan [GitHub Releases](https://github.com/Spoakk/cli/releases/latest) sayfasından işletim sisteminize uygun `.exe` veya binary dosyasını indirip `PATH` (Ortam Değişkenleri) içine ekleyebilirsiniz.

---

## 💻 Kullanım (Komutlar)

Komut satırından direkt çalıştırabilir veya interaktif ortama geçiş yapabilirsiniz.

| Komut | Açıklama |
|----------|----------|
| `spoak update` | CLI ve Backend için en yeni sürümü kontrol eder ve otomatik günceller. |
| `spoak ping <host> [port]` | Hedef sunucuyu pingler, durumu ve MOTD'yi zarif bir panelde gösterir. |
| `spoak player <username>` | Oyuncunun UUID'sini, Skin modelini bulur ve profilini raporlar. |
| `spoak jars versions` | Tüm güncel Minecraft versiyonlarını grid biçiminde listeler. |
| `spoak jars paper <version>` | Paper'ın belirlediğiniz versiyonu için en son build numarasını getirir. |
| `spoak jars purpur <version>` | Purpur'un en güncel versiyon buildini terminale yansıtır. |
| `spoak jars folia <version>` | Folia'nın çok çekirdekli destekli buildlerini gösterir. |
| `spoak jars leaf <version>` | Leaf'in belirlediğiniz sürümü için optimize verileri listeler. |
| `spoak coords nether <x> <z>` | Overworld koordinatlarını Nether'e uyarlar. |
| `spoak structures <seed> [x] [z]` | Dünyadaki en yakın biyom yapılarını hesaplar ve sana uzaklığını söyler. |

---

## 🛠 Kaynak Koddan Derleme

Kendi ortamında derlemek istersen:
```bash
git clone https://github.com/spoakk/cli
cd spoak-cli
cargo build --release
```
Derlenmiş executable `target/release/spoak.exe` altında olacaktır.

## 🔗 İlgili Bağlantılar
- **Web Sitesi:** [spoak.cc](https://spoak.cc)
- **Topluluk:** [Discord Sunucumuz](https://discord.gg/SBbU3rCtGe)
- **Backend:** [spoak-backend (Rust)](https://github.com/spoakk/backend)
