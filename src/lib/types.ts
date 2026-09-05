export type Organization = {
  id: string;
  name: string;
  createdAt: string;
};

export type Topic = {
  id: string;
  organizationId: string;
  name: string;
  createdAt: string;
};

export type MeetingGroup = {
  id: string;
  topicId: string;
  name: string;
  occurredAt: string;
  createdAt: string;
};

export type LibraryTopic = Topic & {
  groups: MeetingGroup[];
};

export type LibraryOrganization = Organization & {
  topics: LibraryTopic[];
};

export type Selection =
  | { kind: "none" }
  | { kind: "organization"; organizationId: string }
  | { kind: "topic"; organizationId: string; topicId: string }
  | { kind: "group"; organizationId: string; topicId: string; groupId: string };
