import type { LibraryOrganization, LibraryTopic, MeetingGroup, Selection } from "./types";

export function findOrg(orgs: LibraryOrganization[], id: string) {
  return orgs.find((org) => org.id === id);
}

export function findTopic(orgs: LibraryOrganization[], organizationId: string, topicId: string) {
  return findOrg(orgs, organizationId)?.topics.find((topic) => topic.id === topicId);
}

export function findGroup(
  orgs: LibraryOrganization[],
  organizationId: string,
  topicId: string,
  groupId: string,
) {
  return findTopic(orgs, organizationId, topicId)?.groups.find((group) => group.id === groupId);
}

export function breadcrumb(orgs: LibraryOrganization[], selection: Selection) {
  if (selection.kind === "none") return [];
  const org = findOrg(orgs, selection.organizationId);
  if (!org) return [];
  const parts = [org.name];
  if (selection.kind === "organization") return parts;
  const topic = org.topics.find((item) => item.id === selection.topicId);
  if (!topic) return parts;
  parts.push(topic.name);
  if (selection.kind === "topic") return parts;
  const group = topic.groups.find((item) => item.id === selection.groupId);
  if (group) parts.push(group.name);
  return parts;
}

export function selectedContext(orgs: LibraryOrganization[], selection: Selection): {
  org?: LibraryOrganization;
  topic?: LibraryTopic;
  group?: MeetingGroup;
} {
  if (selection.kind === "none") return {};
  const org = findOrg(orgs, selection.organizationId);
  if (!org) return {};
  if (selection.kind === "organization") return { org };
  const topic = org.topics.find((item) => item.id === selection.topicId);
  if (!topic) return { org };
  if (selection.kind === "topic") return { org, topic };
  const group = topic.groups.find((item) => item.id === selection.groupId);
  return { org, topic, group };
}
