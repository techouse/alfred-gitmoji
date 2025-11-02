import 'dart:io' show File, exitCode, stdout;

import 'package:alfred_workflow/alfred_workflow.dart';
import 'package:algoliasearch/src/model/hit.dart';
import 'package:algoliasearch/src/model/search_response.dart';
import 'package:args/args.dart' show ArgParser, ArgResults;
import 'package:cli_script/cli_script.dart';

import 'src/env/env.dart' show Env;
import 'src/extensions/string_helpers.dart';
import 'src/models/search_result.dart' show SearchResult;
import 'src/services/algolia_search.dart' show AlgoliaSearch;
import 'src/services/emoji_downloader.dart' show EmojiDownloader;
import 'src/models/user_config_key.dart' show UserConfigKey;

part 'main_helpers.dart';

bool _verbose = false;
bool _update = false;

void main(List<String> arguments) {
  wrapMain(() async {
    try {
      exitCode = 0;

      _workflow.clearItems();

      final ArgParser parser = ArgParser()
        ..addOption('query', abbr: 'q', defaultsTo: '')
        ..addFlag('verbose', abbr: 'v', defaultsTo: false)
        ..addFlag('update', abbr: 'u', defaultsTo: false);
      final ArgResults args = parser.parse(arguments);

      _update = args['update'];
      if (_update) {
        stdout.writeln('Updating workflow...');

        return await _updater.update();
      }

      _verbose = args['verbose'];

      final Map<String, AlfredUserConfiguration>? userDefaults = await _workflow
          .getUserDefaults();

      final AlfredUserConfigurationCheckBox? useFileCache =
          userDefaults?[UserConfigKey.useFileCache.toString()]
              as AlfredUserConfigurationCheckBox?;

      final AlfredUserConfigurationNumberSlider? fileCacheMaxEntries =
          userDefaults?[UserConfigKey.fileCacheMaxEntries.toString()]
              as AlfredUserConfigurationNumberSlider?;

      final AlfredUserConfigurationCheckBox? useAlfredCache =
          userDefaults?[UserConfigKey.useAlfredCache.toString()]
              as AlfredUserConfigurationCheckBox?;

      final AlfredUserConfigurationNumberSlider? cacheTimeToLive =
          userDefaults?[UserConfigKey.cacheTtl.toString()]
              as AlfredUserConfigurationNumberSlider?;

      final String query = args['query']
          .replaceAll(RegExp(r'\s+'), ' ')
          .trim()
          .toLowerCase();

      if (_verbose) stdout.writeln('Query: "$query"');

      if (useAlfredCache?.value ?? false) {
        _workflow.useAutomaticCache = true;
      } else if (useFileCache?.value ?? false) {
        _workflow.cacheKey = query.isNotEmpty ? query : 'ALL_GITMOJIS'.md5hex;
        _workflow.maxCacheEntries =
            fileCacheMaxEntries?.value ?? fileCacheMaxEntries?.defaultValue;
      }

      _workflow.cacheTimeToLive = cacheTimeToLive?.value;

      if ((await _workflow.getItems()).isEmpty) {
        await _performSearch(query.isNotEmpty ? query : '');
      }
    } on FormatException catch (err) {
      exitCode = 2;
      _workflow.addItem(AlfredItem(title: err.toString()));
    } catch (err) {
      exitCode = 1;
      _workflow.addItem(AlfredItem(title: err.toString()));
      if (_verbose) rethrow;
    } finally {
      if (!_update) {
        if (await _updater.updateAvailable()) {
          _workflow.run(addToBeginning: updateItem);
        } else {
          _workflow.run();
        }
      }
    }
  });
}
