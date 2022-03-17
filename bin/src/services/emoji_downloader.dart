import 'dart:io' show Directory, File, Platform;

import 'package:http/http.dart' as http show get, Response;
import 'package:path/path.dart' as path show dirname, join;

class EmojiDownloader {
  const EmojiDownloader({
    required this.emoji,
    String? directoryPath,
  }) : _directoryPath = directoryPath;

  final String emoji;
  final String? _directoryPath;

  Future<File?> downloadImage() async {
    final String fileName = '${emoji.runes.first.toRadixString(16)}.png';
    final String filePath =
        _directoryPath != null && await Directory(_directoryPath!).exists()
            ? path.join(_directoryPath!, fileName)
            : path.join(
                path.dirname(Platform.script.toFilePath()),
                'image_cache',
                fileName,
              );
    final File file = File(filePath);

    if (!await file.exists()) {
      await file.create(recursive: true);

      final http.Response response = await http.get(
        Uri.https(
          'raw.githubusercontent.com',
          '/joypixels/emoji-assets/master/png/128/$fileName',
        ),
      );

      if (response.statusCode < 400) {
        file.writeAsBytes(response.bodyBytes);
      } else {
        return null;
      }
    }

    return file;
  }
}
