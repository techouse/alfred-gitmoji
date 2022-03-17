class SearchResult {
  const SearchResult({
    required this.objectID,
    required this.emoji,
    required this.entity,
    required this.code,
    required this.description,
    required this.name,
    this.semver,
  });

  final String objectID;
  final String emoji;
  final String entity;
  final String code;
  final String description;
  final String name;
  final String? semver;

  static const List<String> attributesToRetrieve = [
    'emoji',
    'entity',
    'code',
    'description',
    'name',
    'semver',
  ];

  SearchResult.fromJson(Map<String, dynamic> json)
      : objectID = json['objectID'] as String,
        emoji = json['emoji'] as String,
        entity = json['entity'] as String,
        code = json['code'] as String,
        description = json['description'] as String,
        name = json['name'] as String,
        semver = json['semver'] as String?;

  Map<String, dynamic> toJson() => {
        'emoji': emoji,
        'entity': entity,
        'code': code,
        'description': description,
        'name': name,
        'semver': semver,
      };
}
