import 'package:recase/recase.dart';

enum UserConfigKey {
  useAlfredCache,
  useFileCache,
  cacheTtl,
  fileCacheMaxEntries;

  @override
  String toString() => name.snakeCase;
}
