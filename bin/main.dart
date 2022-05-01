import 'dart:io' show File, exitCode, stdout;
import 'dart:convert' show HtmlEscape;

import 'package:alfred_workflow/alfred_workflow.dart'
    show
        AlfredCache,
        AlfredItem,
        AlfredItemIcon,
        AlfredItemText,
        AlfredItems,
        AlfredUpdater,
        AlfredWorkflow;
import 'package:algolia/algolia.dart' show AlgoliaQuerySnapshot;
import 'package:args/args.dart' show ArgParser, ArgResults;
import 'package:stash/stash_api.dart' show CreatedExpiryPolicy;

import 'src/constants/config.dart' show Config;
import 'src/models/search_result.dart' show SearchResult;
import 'src/services/algolia_search.dart' show AlgoliaSearch;
import 'src/services/emoji_downloader.dart' show EmojiDownloader;

const HtmlEscape htmlEscape = HtmlEscape();
final AlfredWorkflow workflow = AlfredWorkflow(
  cache: AlfredCache<AlfredItems>(
    fromEncodable: (Map<String, dynamic> json) => AlfredItems.fromJson(json),
    maxEntries: 1024,
    expiryPolicy: const CreatedExpiryPolicy(
      Duration(days: 7),
    ),
  ),
);
final AlfredUpdater updater = AlfredUpdater(
  githubRepositoryUrl: Config.githubRepositoryUrl,
  currentVersion: Config.version,
  updateInterval: Duration(days: 7),
);
bool verbose = false;
bool update = false;

void main(List<String> arguments) async {
  try {
    exitCode = 0;

    workflow.clearItems();

    final ArgParser parser = ArgParser()
      ..addOption('query', abbr: 'q', defaultsTo: '')
      ..addFlag('verbose', abbr: 'v', defaultsTo: false)
      ..addFlag('update', abbr: 'u', defaultsTo: false);
    final ArgResults args = parser.parse(arguments);

    update = args['update'];
    if (update) {
      stdout.writeln('Updating workflow...');

      return await updater.update();
    }

    verbose = args['verbose'];

    final String query =
        args['query'].replaceAll(RegExp(r'\s+'), ' ').trim().toLowerCase();

    if (verbose) stdout.writeln('Query: "$query"');

    if (query.isEmpty) {
      _showPlaceholder();
    } else {
      workflow.cacheKey = query;
      if (await workflow.getItems() == null) {
        await _performSearch(query);
      }
    }
  } on FormatException catch (err) {
    exitCode = 2;
    workflow.addItem(AlfredItem(title: err.toString()));
  } catch (err) {
    exitCode = 1;
    workflow.addItem(AlfredItem(title: err.toString()));
    if (verbose) rethrow;
  } finally {
    if (!update) {
      if (await updater.updateAvailable()) {
        workflow.run(addToBeginning: updateItem);
      } else {
        workflow.run();
      }
    }
  }
}

const updateItem = AlfredItem(
  title: 'Auto-Update available!',
  subtitle: 'Press <enter> to auto-update to a new version of this workflow.',
  arg: 'update:workflow',
  match:
      'Auto-Update available! Press <enter> to auto-update to a new version of this workflow.',
  icon: AlfredItemIcon(path: 'alfredhatcog.png'),
  valid: true,
);

void _showPlaceholder() {
  workflow.addItem(
    const AlfredItem(
      title: 'Search for gitmojis ...',
      icon: AlfredItemIcon(path: 'icon.png'),
    ),
  );
}

Future<void> _performSearch(String query) async {
  final AlgoliaQuerySnapshot snapshot = await AlgoliaSearch.query(query);

  if (snapshot.nbHits > 0) {
    final AlfredItems items = AlfredItems(
      await Future.wait(snapshot.hits
          .map((snapshot) => SearchResult.fromJson(snapshot.data))
          .map((result) async {
        final File? image = await EmojiDownloader(
          emoji: result.emoji,
        ).downloadImage();

        return AlfredItem(
          uid: result.objectID,
          title: result.code,
          subtitle: result.description,
          arg: result.code,
          match: '${result.name} ${result.description}',
          text: AlfredItemText(
            copy: result.code,
            largeType: result.code,
          ),
          icon: AlfredItemIcon(
            path: image != null ? image.absolute.path : 'question.png',
          ),
          valid: true,
        );
      }).toList()),
    );
    workflow.addItems(items.items);
  } else {
    workflow.addItem(
      AlfredItem(
        title: 'No matching gitmoji found',
        icon: AlfredItemIcon(
          path: 'question.png',
        ),
        valid: false,
      ),
    );
  }
}
