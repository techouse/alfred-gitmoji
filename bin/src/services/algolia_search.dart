import 'package:algoliasearch/algoliasearch_lite.dart';

import '../env/env.dart';
import '../models/search_result.dart';

class AlgoliaSearch {
  const AlgoliaSearch._();

  static final SearchClient _client = SearchClient(
    appId: Env.algoliaApplicationId,
    apiKey: Env.algoliaSearchOnlyApiKey,
  );

  static Future<SearchResponse> query(String queryString) =>
      _client.searchIndex(
        request: SearchForHits(
          indexName: Env.algoliaSearchIndex,
          query: queryString,
          attributesToRetrieve: SearchResult.attributesToRetrieve,
          page: 0,
          hitsPerPage: 100,
        ),
      );

  static void dispose() => _client.dispose();
}
