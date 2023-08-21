import type { NextPage } from 'next';
import {
  Button,
  Callout,
  Card,
  Dialog,
  DialogBody,
  DialogFooter,
  Elevation,
  H3,
  H5,
  HTMLTable,
  InputGroup,
  OverlayToaster,
  Position,
  Toast,
} from '@blueprintjs/core';
import { useSession } from 'next-auth/react';
import useTranslation from 'next-translate/useTranslation';
import { useState } from 'react';
import useSWR from 'swr';
import { apiTokenFetcher, createToken, deleteToken } from '../utils/api';
import { Token } from '../interfaces/Token';

const Tokens: NextPage = () => {
  const { t } = useTranslation('common');
  const [isDialogOpen, setIsDialogOpen] = useState(false);
  const [tokenName, setTokenName] = useState('');
  const [createdTokens, setCreatedTokens] = useState<Token[]>([]);
  const [creationError, setCreationError] = useState<string | null>(null);
  const [deletionError, setDeletionError] = useState<string | null>(null);
  const [deletionSuccess, setDeletionSuccess] = useState<string | null>(null);
  const session = useSession();
  const email = session?.data?.user?.email;

  const {
    data: fetchedTokens,
    error,
    mutate,
  } = useSWR('/api/v1/token', apiTokenFetcher);

  return (
    <div>
      <OverlayToaster position={Position.TOP}>
        {creationError && (
          <Toast timeout={10000} intent="danger" message={creationError} />
        )}
        {deletionError && (
          <Toast timeout={10000} intent="danger" message={deletionError} />
        )}
        {deletionSuccess && (
          <Toast timeout={10000} intent="success" message={deletionSuccess} />
        )}
      </OverlayToaster>
      <main className="flex flex-row justify-center">
        <div className="flex flex-col items-strech w-full max-w-2xl">
          <div className="flex flex-row mt-6 items-end justify-between">
            <H3 className="m-0">{t('tokens')}</H3>
            <Button
              intent="primary"
              large
              onClick={() => {
                setIsDialogOpen(true);
              }}
            >
              {t('createToken')}
            </Button>
            <Dialog
              isOpen={isDialogOpen}
              title={t('createToken')}
              icon="info-sign"
              onClose={() => {
                setIsDialogOpen(false);
              }}
            >
              <DialogBody>
                <InputGroup
                  placeholder={t('tokenName')}
                  value={tokenName}
                  onChange={(event) => {
                    setTokenName(event.target.value);
                  }}
                />
              </DialogBody>
              <DialogFooter
                actions={
                  <Button
                    disabled={tokenName === ''}
                    intent="primary"
                    text={t('create')}
                    onClick={async () => {
                      if (email) {
                        try {
                          const newToken = await createToken({
                            name: tokenName,
                            email,
                          });
                          setCreatedTokens([...createdTokens, newToken]);
                          setIsDialogOpen(false);
                        } catch (err) {
                          setCreationError(t('failedToCreateToken'));
                        }
                      } else {
                        setCreationError(t('userEmailMissing'));
                      }
                      setTokenName('');
                    }}
                  />
                }
              />
            </Dialog>
          </div>
          <div className="flex flex-row justify-center">
            <div className="flex flex-col w-full max-w-sm">
              {createdTokens
                .sort(
                  (a, b) => Date.parse(b.createdAt) - Date.parse(a.createdAt)
                )
                .map((token) => (
                  <Card
                    className="mt-10 flex flex-row justify-between items-center"
                    interactive={false}
                    elevation={Elevation.TWO}
                  >
                    <H5 className="m-0 text-ellipsis">{token.name}</H5>

                    <Button
                      icon="clipboard"
                      minimal
                      onClick={() => {
                        navigator.clipboard.writeText(token.token);
                      }}
                    >
                      {t('copyValue')}
                    </Button>
                  </Card>
                ))}
            </div>
          </div>
          <div className="mt-10 flex flex-col justify-center">
            {error ? (
              <Callout intent="danger" title={t('failedToFetchTokens')} />
            ) : (
              <HTMLTable striped bordered interactive>
                <thead>
                  <tr>
                    <th>{t('name')}</th>
                    <th>{t('createdAt')}</th>
                    <th> </th>
                  </tr>
                </thead>
                <tbody>
                  {(fetchedTokens ?? [])
                    .sort(
                      (a, b) =>
                        Date.parse(b.createdAt) - Date.parse(a.createdAt)
                    )
                    .map((token) => (
                      <tr key={token.id}>
                        <td className="text-ellipsis">{token.name}</td>
                        <td>{new Date(token.createdAt).toLocaleString()}</td>
                        <td className="flex justify-left">
                          <Button
                            intent="danger"
                            icon="trash"
                            small
                            onClick={async () => {
                              try {
                                await deleteToken(token.id);
                                setDeletionSuccess(t('tokenDeleted'));
                                mutate();
                              } catch (err) {
                                setDeletionError(t('failedToDeleteToken'));
                              }
                            }}
                          >
                            {t('delete')}
                          </Button>
                        </td>
                      </tr>
                    ))}
                </tbody>
              </HTMLTable>
            )}
          </div>
        </div>
      </main>
    </div>
  );
};

export default Tokens;
