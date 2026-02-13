insert into storage.buckets (id, name, public, file_size_limit, allowed_mime_types)
values (
  'demo_images',
  'demo_images',
  false,
  5242880,
  array['image/jpeg', 'image/png', 'image/webp', 'image/gif']::text[]
)
on conflict (id) do update
set
  public = excluded.public,
  file_size_limit = excluded.file_size_limit,
  allowed_mime_types = excluded.allowed_mime_types;

drop policy if exists demo_images_select_own on storage.objects;
create policy demo_images_select_own
  on storage.objects
  for select
  to authenticated
  using (
    bucket_id = 'demo_images'
    and (storage.foldername(name))[1] = auth.uid()::text
  );

drop policy if exists demo_images_insert_own on storage.objects;
create policy demo_images_insert_own
  on storage.objects
  for insert
  to authenticated
  with check (
    bucket_id = 'demo_images'
    and (storage.foldername(name))[1] = auth.uid()::text
  );

drop policy if exists demo_images_update_own on storage.objects;
create policy demo_images_update_own
  on storage.objects
  for update
  to authenticated
  using (
    bucket_id = 'demo_images'
    and (storage.foldername(name))[1] = auth.uid()::text
  )
  with check (
    bucket_id = 'demo_images'
    and (storage.foldername(name))[1] = auth.uid()::text
  );

drop policy if exists demo_images_delete_own on storage.objects;
create policy demo_images_delete_own
  on storage.objects
  for delete
  to authenticated
  using (
    bucket_id = 'demo_images'
    and (storage.foldername(name))[1] = auth.uid()::text
  );
