import 'dart:io';

import 'package:http/http.dart' as http;
import 'package:archive/archive_io.dart';

Future<void> load() async {
  final pathComponents = Platform.resolvedExecutable.split(Platform.pathSeparator);
  pathComponents.removeLast();
  pathComponents.removeLast();
  pathComponents.removeLast();
  pathComponents.removeLast();
  pathComponents.add('internal');
  pathComponents.add('engine.version');

  final path = pathComponents.join(Platform.pathSeparator);
  final engineHash = (await File(path).readAsString()).trim();

  final Uri uri;

  if (Platform.isWindows) {
    uri = Uri.https(
        'storage.googleapis.com', '/flutter_infra_release/flutter/$engineHash/windows-x64/windows-x64-embedder.zip');
  } else if (Platform.isMacOS) {
    uri = Uri.https('storage.googleapis.com',
        '/flutter_infra_release/flutter/$engineHash/darwin-x64/FlutterEmbedder.framework.zip');
  } else if (Platform.isLinux) {
    uri = Uri.https(
        'storage.googleapis.com', '/flutter_infra_release/flutter/$engineHash/linux-x64/linux-x64-embedder.zip');
  } else {
    throw "Platform not supported!";
  }

  print('Downloading $uri...');
  final response = await http.get(uri);

  if (response.statusCode == 200) {
    print('Decompressing $uri...');
    final archive = ZipDecoder().decodeBuffer(InputStream(response.bodyBytes), verify: true);
    Directory('out').createSync();
    for (final file in archive.files) {
      // If it's a file and not a directory
      if (file.isFile) {
        final fileName = Uri.file(file.name).pathSegments.last;
        if (fileName.contains('..')) {
          print('Invalid file name ${file.name}, skipped');
          continue;
        }
        print('Writing file $fileName...');
        final outputStream = OutputFileStream('out/$fileName');
        file.writeContent(outputStream);
        outputStream.close();
      }
    }
    print('Done!');
  } else {
    throw "Fetch failed: $response";
  }
}
