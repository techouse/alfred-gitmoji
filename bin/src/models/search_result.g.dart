// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'search_result.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

SearchResult _$SearchResultFromJson(Map<String, dynamic> json) => SearchResult(
  objectID: json['objectID'] as String,
  emoji: json['emoji'] as String,
  entity: json['entity'] as String,
  code: json['code'] as String,
  description: json['description'] as String,
  name: json['name'] as String,
  semver: json['semver'] as String?,
);

Map<String, dynamic> _$SearchResultToJson(SearchResult instance) =>
    <String, dynamic>{
      'objectID': instance.objectID,
      'emoji': instance.emoji,
      'entity': instance.entity,
      'code': instance.code,
      'description': instance.description,
      'name': instance.name,
      'semver': instance.semver,
    };
