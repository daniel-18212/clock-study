# Clock Study

Aplicativo desktop de foco para Windows 11. Pomodoro, cron�metro, temporizador e janela flutuante sempre acima das outras janelas.

## Compilar no Windows

Instale o [Rust com rustup](https://rustup.rs/) e o [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/). No instalador do Visual Studio, selecione **Desenvolvimento para desktop com C++**, com MSVC e SDK do Windows. Use Node.js 22 LTS e o WebView2 do Windows. Veja os [pr�-requisitos do Tauri](https://v2.tauri.app/start/prerequisites/).

```powershell
npm install
npm run tauri dev
npm run tauri build -- --no-bundle
```

O execut�vel de compila��o fica em `src-tauri/target/release/clock-study.exe`. O Windows pode bloquear a execu��o desse bin�rio sem assinatura quando o Controle Inteligente de Aplicativos estiver ativo.

## Gerar e publicar o MSIX

1. Crie o produto **MSIX/PWA** no [Partner Center](https://partner.microsoft.com/dashboard). Abra a se��o **Identidade do produto** e copie o nome da identidade, o publicador e o nome de exibi��o do publicador.
2. No GitHub, em **Settings  Secrets and variables  Actions  Variables**, adicione `STORE_IDENTITY_NAME`, `STORE_PUBLISHER` e `STORE_PUBLISHER_DISPLAY_NAME`.
3. Rode o workflow **Clock Study Windows** com **Preparar MSIX para submiss�o � Microsoft Store** marcado. Tamb�m � poss�vel empacotar localmente com o script `packaging/windows/pack-msix.ps1`, passando esses tr�s campos pelos par�metros.
4. Envie o MSIX para certifica��o no Partner Center. A Store assina o pacote durante a publica��o. Valide a instala��o da vers�o publicada com o Controle Inteligente de Aplicativos ativo.

[Documenta��o da Microsoft sobre assinatura pela Store](https://learn.microsoft.com/windows/apps/publish/faq/get-started-with-the-microsoft-store)

## Verificar no Windows 11

- Inicie um Pomodoro, pause, retome e reinicie. Edite as tr�s dura��es e o n�mero de blocos.
- Defina um temporizador de 10 segundos, minimize o app e confira a notifica��o ao terminar.
- Registre voltas no cron�metro e abra a lista. Reinicie e confira que a lista foi limpa.
- Acione **Flutuar**, abra outro aplicativo e confira se o rel�gio permanece acima. Arraste a janela e redimensione pelos cantos. Volte � janela principal com a contagem ativa.
- Altere os tamanhos e os tempos, feche e reabra. Verifique se as prefer�ncias foram recuperadas.

