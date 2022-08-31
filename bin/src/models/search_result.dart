import 'package:json_annotation/json_annotation.dart';

part 'search_result.g.dart';

@JsonSerializable()
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

  factory SearchResult.fromJson(Map<String, dynamic> json) =>
      _$SearchResultFromJson(json);

  Map<String, dynamic> toJson() => _$SearchResultToJson(this);
}
