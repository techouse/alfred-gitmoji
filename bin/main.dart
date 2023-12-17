import 'dart:io' show File, exitCode, stdout;

import 'package:alfred_workflow/alfred_workflow.dart'
    show
        AlfredCache,
        AlfredItem,
        AlfredItemIcon,
        AlfredItemMod,
        AlfredItemModKey,
        AlfredItemText,
        AlfredItems,
        AlfredUpdater,
        AlfredWorkflow;
import 'package:algoliasearch/src/model/hit.dart';
import 'package:algoliasearch/src/model/search_response.dart';
import 'package:args/args.dart' show ArgParser, ArgResults;
import 'package:cli_script/cli_script.dart';
import 'package:stash/stash_api.dart' show CreatedExpiryPolicy;

import 'src/env/env.dart' show Env;
import 'src/extensions/string_helpers.dart';
import 'src/models/search_result.dart' show SearchResult;
import 'src/services/algolia_search.dart' show AlgoliaSearch;
import 'src/services/emoji_downloader.dart' show EmojiDownloader;

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

      final String query =
          args['query'].replaceAll(RegExp(r'\s+'), ' ').trim().toLowerCase();

      if (_verbose) stdout.writeln('Query: "$query"');

      _workflow.cacheKey = query.isNotEmpty ? query : 'ALL_GITMOJIS'.md5hex;
      if (await _workflow.getItems() == null) {
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
