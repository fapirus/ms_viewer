import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:ms_viewer_platform_interface/ms_viewer_platform_interface.dart';

import 'src/engine/demo_engine_platform.dart';
import 'src/demo_home_page.dart';

void main() {
  runApp(DemoApp());
}

class DemoApp extends StatelessWidget {
  DemoApp({
    super.key,
    this.platform,
    this.assetBundle,
  });

  final MsViewerPlatform? platform;
  final AssetBundle? assetBundle;

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      debugShowCheckedModeBanner: false,
      title: 'MS Viewer Demo',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: Colors.blue),
        useMaterial3: true,
        scaffoldBackgroundColor: const Color(0xFFF3F7FC),
      ),
      home: DemoHomePage(
        viewerPlatform: platform ?? DemoEnginePlatform(),
        assetBundle: assetBundle ?? rootBundle,
      ),
    );
  }
}
