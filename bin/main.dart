import 'dart:io' show File, exitCode, stdout;
import 'dart:convert';

import 'package:alfred_workflow/alfred_workflow.dart'
    show
        AlfredItem,
        AlfredItemIcon,
        AlfredItemText,
        AlfredItems,
        AlfredWorkflow;
import 'package:algolia/algolia.dart' show AlgoliaQuerySnapshot;
import 'package:args/args.dart' show ArgParser, ArgResults;

import 'src/models/search_result.dart' show SearchResult;
import 'src/services/algolia_search.dart' show AlgoliaSearch;
import 'src/services/emoji_downloader.dart';

const HtmlEscape htmlEscape = HtmlEscape();

final AlfredWorkflow workflow = AlfredWorkflow();

bool verbose = false;

void main(List<String> arguments) async {
  try {
    exitCode = 0;

    workflow.clearItems();

    final ArgParser parser = ArgParser()
      ..addOption('query', abbr: 'q', mandatory: true)
      ..addFlag('verbose', abbr: 'v', defaultsTo: false);
    final ArgResults args = parser.parse(arguments);

    verbose = args['verbose'];

    final String query = args['query'].replaceAll(RegExp(r'\s+'), ' ').trim();

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
    if (verbose) {
      rethrow;
    }
  } finally {
    workflow.run();
  }
}

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
          icon:
              image != null ? AlfredItemIcon(path: image.absolute.path) : null,
          valid: true,
        );
      }).toList()),
    );
    workflow.addItems(items.items);
  } else {
    final Uri url = Uri.https(
      'www.google.com',
      '/search',
      {'q': 'gitmoji $query'},
    );

    workflow.addItem(
      AlfredItem(
        title: 'No matching answers found',
        subtitle: 'Shall I try and search Google?',
        arg: url.toString(),
        text: AlfredItemText(
          copy: url.toString(),
        ),
        quickLookUrl: url.toString(),
        icon: AlfredItemIcon(path: 'google.png'),
        valid: true,
      ),
    );
  }
}
