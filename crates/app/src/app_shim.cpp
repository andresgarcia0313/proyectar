// Application bootstrap that cxx-qt-lib 0.10 does not cover: QApplication, the Basic style,
// the QML engine and the test hooks (auto quit and screenshot).
#include <QtCore/QTimer>
#include <QtCore/QUrl>
#include <QtGui/QGuiApplication>
#include <QtGui/QIcon>
#include <QtQml/QQmlApplicationEngine>
#include <QtQuick/QQuickWindow>
#include <QtQuickControls2/QQuickStyle>
#include <QtWidgets/QApplication>

namespace {
int app_argc = 0;

// PROYECTAR_CAPTURA=<file.png>: save the first window after PROYECTAR_CAPTURA_MS (1500).
// PROYECTAR_CAPTURA_VENTANA=<objectName>: show that secondary window and save it instead.
QQuickWindow *window_to_grab(QQmlApplicationEngine &engine) {
    const QString name = qEnvironmentVariable("PROYECTAR_CAPTURA_VENTANA");
    if (name.isEmpty()) {
        return engine.rootObjects().isEmpty()
                   ? nullptr : qobject_cast<QQuickWindow *>(engine.rootObjects().first());
    }
    for (QWindow *window : QGuiApplication::topLevelWindows()) {
        if (window->objectName() == name) return qobject_cast<QQuickWindow *>(window);
    }
    return nullptr;
}

void schedule_screenshot(QQmlApplicationEngine &engine) {
    const QString file = qEnvironmentVariable("PROYECTAR_CAPTURA");
    if (file.isEmpty()) return;
    const int delay = qEnvironmentVariableIsSet("PROYECTAR_CAPTURA_MS")
                          ? qEnvironmentVariableIntValue("PROYECTAR_CAPTURA_MS") : 1500;
    QTimer::singleShot(delay / 2, qApp, [&engine] {
        if (auto *window = window_to_grab(engine)) window->show();
    });
    QTimer::singleShot(delay, qApp, [&engine, file] {
        if (auto *window = window_to_grab(engine)) window->grabWindow().save(file);
    });
}
}  // namespace

extern "C" void proyectar_app_new(int argc, char **argv) {
    app_argc = argc;
    auto *app = new QApplication(app_argc, argv);
    app->setApplicationName(QStringLiteral("Proyectar"));
    app->setOrganizationName(QStringLiteral("ingeniumcodex"));
    app->setOrganizationDomain(QStringLiteral("ingeniumcodex.com"));
    app->setDesktopFileName(QStringLiteral("com.ingeniumcodex.proyectar"));
    QQuickStyle::setStyle(QStringLiteral("Basic"));
}

extern "C" int proyectar_app_run() {
    QApplication::setWindowIcon(QIcon(QStringLiteral(":/qt/qml/ProyectarApp/assets/proyectar.svg")));
    int code = 0;
    {
        QQmlApplicationEngine engine;
        engine.addImportPath(QStringLiteral("qrc:/qt/qml"));
        QObject::connect(
            &engine, &QQmlApplicationEngine::objectCreationFailed, qApp,
            [] { QCoreApplication::exit(1); }, Qt::QueuedConnection);
        // Smoke tests and CI: quit on their own after the given milliseconds.
        if (qEnvironmentVariableIsSet("PROYECTAR_AUTOQUIT_MS")) {
            QTimer::singleShot(qEnvironmentVariableIntValue("PROYECTAR_AUTOQUIT_MS"), qApp,
                               &QCoreApplication::quit);
        }
        engine.load(QUrl(QStringLiteral("qrc:/qt/qml/ProyectarApp/qml/Main.qml")));
        schedule_screenshot(engine);
        code = QApplication::exec();
    }
    QCoreApplication::sendPostedEvents(nullptr, QEvent::DeferredDelete);
    delete qApp;
    return code;
}
