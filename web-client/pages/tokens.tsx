import type { NextPage } from 'next';
import {
  Button,
  Card,
  Dialog,
  DialogBody,
  DialogFooter,
  Elevation,
  H3,
  H5,
  HTMLTable,
  InputGroup,
} from '@blueprintjs/core';
import useTranslation from 'next-translate/useTranslation';
import { useState } from 'react';
import useSWR from 'swr';
import { apiTokenFetcher, createToken } from '../utils/api';
import { Token } from '../interfaces/Token';

const Tokens: NextPage = () => {
  const { t } = useTranslation('common');
  const [isDialogOpen, setIsDialogOpen] = useState(false);
  const [tokenName, setTokenName] = useState('');
  const [createdTokens, setCreatedTokens] = useState<Token[]>([]);

  const { data: fetchedTokens, error } = useSWR(
    '/api/v1/token',
    apiTokenFetcher
  );

  if (error || !fetchedTokens) {
    return <div>{t('failedToFetchTokens')} </div>;
  }

  return (
    <div>
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
                      const newToken = await createToken({
                        name: tokenName,
                      });
                      setCreatedTokens([...createdTokens, newToken]);
                      setIsDialogOpen(false);
                    }}
                  />
                }
              />
            </Dialog>
          </div>
          {createdTokens
            .sort((a, b) => Date.parse(b.createdAt) - Date.parse(a.createdAt))
            .map((token) => (
              <Card
                className="mt-10"
                interactive={false}
                elevation={Elevation.TWO}
              >
                <H5>{token.name}</H5>

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
          <HTMLTable striped bordered className="mt-10">
            <thead>
              <tr>
                <th>{t('name')}</th>
                <th>{t('createdAt')}</th>
                <th> </th>
              </tr>
            </thead>
            <tbody>
              {fetchedTokens
                .sort(
                  (a, b) => Date.parse(b.createdAt) - Date.parse(a.createdAt)
                )
                .map((token) => (
                  <tr key={token.id}>
                    <td>{token.name}</td>
                    <td>{new Date(token.createdAt).toLocaleString()}</td>
                    <td>
                      <Button intent="danger" icon="trash" small>
                        {t('delete')}
                      </Button>
                    </td>
                  </tr>
                ))}
            </tbody>
          </HTMLTable>
        </div>
      </main>
    </div>
  );
};

export default Tokens;
