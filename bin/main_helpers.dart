part of 'main.dart';

final AlfredWorkflow _workflow = AlfredWorkflow()
  ..disableAlfredSmartResultOrdering = true;

final AlfredUpdater _updater = AlfredUpdater(
  githubRepositoryUrl: Env.githubRepositoryUrl,
  currentVersion: Env.appVersion,
  updateInterval: Duration(days: 7),
);

const updateItem = AlfredItem(
  title: 'Auto-Update available!',
  subtitle: 'Press <enter> to auto-update to a new version of this workflow.',
  arg: 'update:workflow',
  match:
      'Auto-Update available! Press <enter> to auto-update to a new version of this workflow.',
  icon: AlfredItemIcon(path: 'alfredhatcog.png'),
  valid: true,
);

Future<void> _performSearch(String query) async {
  try {
    final SearchResponse searchResponse = await AlgoliaSearch.query(query);

    if ((searchResponse.nbHits ?? 0) > 0) {
      final AlfredItems items = AlfredItems(
        await Future.wait(searchResponse.hits
            .map((Hit hit) => SearchResult.fromJson(
                <String, dynamic>{...hit, 'objectID': hit.objectID}))
            .map((SearchResult result) async {
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
            mods: {
              {AlfredItemModKey.alt}: AlfredItemMod(
                subtitle: 'Copy "${result.emoji}" to clipboard',
                arg: result.emoji,
                icon: AlfredItemIcon(
                    path: image?.absolute.path ?? 'question.png'),
              ),
              {AlfredItemModKey.shift}: AlfredItemMod(
                subtitle:
                    'Copy Python source of "${result.emoji}" to clipboard',
                arg: 'u"\\U000'
                    '${result.emoji.runes.first.toRadixString(16).toUpperCase()}'
                    '${result.emoji.runes.toList().sublist(1).map(
                          (int i) => '\\u${i.toRadixString(16).toUpperCase()}',
                        ).join()}"',
                icon: AlfredItemIcon(
                    path: image?.absolute.path ?? 'question.png'),
              ),
              {AlfredItemModKey.ctrl}: AlfredItemMod(
                subtitle: 'Copy HTML Entity of "${result.emoji}" to clipboard',
                arg: result.emoji.runes
                    .map((int i) => '&#x${i.toRadixString(16)};')
                    .join(),
                icon: AlfredItemIcon(
                    path: image?.absolute.path ?? 'question.png'),
              ),
              {AlfredItemModKey.ctrl, AlfredItemModKey.shift}: AlfredItemMod(
                subtitle:
                    'Copy formal Unicode notation of "${result.emoji}" to clipboard',
                arg: result.emoji.runes
                    .map((int i) => 'U+${i.toRadixString(16).toUpperCase()}')
                    .join(', '),
                icon: AlfredItemIcon(
                    path: image?.absolute.path ?? 'question.png'),
              ),
            },
            valid: true,
          );
        }).toList()),
      );
      _workflow.addItems(items.items);
    } else {
      _workflow.addItem(
        AlfredItem(
          title: 'No matching gitmoji found',
          icon: AlfredItemIcon(
            path: 'question.png',
          ),
          valid: false,
        ),
      );
    }
  } finally {
    AlgoliaSearch.dispose();
  }
}
