import 'package:algolia/algolia.dart'
    show Algolia, AlgoliaQuery, AlgoliaQuerySnapshot;

import '../constants/config.dart';
import '../models/search_result.dart';

class AlgoliaSearch {
  AlgoliaSearch._();

  static final Algolia _algolia = Algolia.init(
    applicationId: Config.algoliaApplicationId,
    apiKey: Config.algoliaSearchOnlyApiKey,
  );

  static Future<AlgoliaQuerySnapshot> query(String queryString) async {
    final AlgoliaQuery query = _algolia.instance
        .index(Config.algoliaSearchIndex)
        .query(queryString)
        .setAttributesToRetrieve(SearchResult.attributesToRetrieve)
        .setPage(0)
        .setHitsPerPage(20);

    return await query.getObjects();
  }
}
