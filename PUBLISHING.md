# Crates.io'ya Yükleme Talimatları

Bu belge, repo-analyzer projesini crates.io'ya yükleme adımlarını açıklar.

## Ön Hazırlık

1. Cargo.toml dosyasındaki bilgileri kendi bilgilerinizle güncelleyin:
   - `authors` alanını kendi adınız ve e-posta adresinizle değiştirin
   - `repository` alanını kendi GitHub repo URL'nizle değiştirin
   - Diğer meta verileri gerektiği gibi düzenleyin

2. LICENSE dosyasındaki telif hakkı bilgisini kendi adınızla güncelleyin.

3. README.md dosyasındaki tüm "yourusername" referanslarını kendi GitHub kullanıcı adınızla değiştirin.

## crates.io Hesabı

1. Eğer henüz bir crates.io hesabınız yoksa, https://crates.io adresinden kaydolun.

2. Hesabınızı oluşturduktan sonra, API token'ınızı alın:
   - Crates.io'da oturum açın
   - Sağ üst köşedeki kullanıcı adınıza tıklayın
   - "Account Settings" seçeneğini seçin
   - "API Tokens" bölümünde yeni bir token oluşturun

3. Cargo'ya API token'ınızı kaydedin:
   ```bash
   cargo login YOUR_API_TOKEN
   ```

## Paketi Hazırlama ve Yükleme

1. Paketinizi test edin:
   ```bash
   cargo package --allow-dirty
   ```

2. Paketinizi crates.io'ya yükleyin:
   ```bash
   cargo publish --allow-dirty
   ```

   Not: `--allow-dirty` bayrağı, git'e commit edilmemiş değişiklikleri içeren dosyaları da pakete dahil eder. Gerçek bir yayında, tüm değişiklikleri commit etmeniz ve bu bayrağı kullanmamanız önerilir.

## Yükleme Sonrası

1. Paketinizin crates.io'da görünüp görünmediğini kontrol edin:
   ```
   https://crates.io/crates/repo-analyzer
   ```

2. Paketinizi cargo ile kurmayı deneyin:
   ```bash
   cargo install repo-analyzer
   ```

## Yeni Sürüm Yayınlama

Yeni bir sürüm yayınlamak istediğinizde:

1. Cargo.toml dosyasındaki `version` numarasını artırın (örneğin, "0.1.0" -> "0.1.1")
2. Değişiklikleri commit edin
3. Yeni sürümü yayınlayın:
   ```bash
   cargo publish
   ```

## Önemli Notlar

- Bir paketi crates.io'ya yükledikten sonra, o sürümü asla silemezsiniz. Yeni sürümler yayınlayabilirsiniz, ancak eski sürümler her zaman erişilebilir kalır.
- Paket adları benzersiz olmalıdır. Eğer "repo-analyzer" adı zaten alınmışsa, farklı bir ad seçmeniz gerekecektir.
- Paketinizin bağımlılıklarının da crates.io'da mevcut olması gerekir. 